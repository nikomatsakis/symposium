//! Integration tests for the initialization sequence and proxy capability handshake.
//!
//! These tests verify that:
//! 1. Single-component chains do NOT receive the proxy capability offer
//! 2. Multi-component chains: first component(s) receive proxy capability offer
//! 3. Proxy components must accept the capability or initialization fails
//! 4. Last component (agent) never receives proxy capability offer

use agent_client_protocol::{InitializeRequest, InitializeResponse};
use conductor::component::{ComponentProvider, MockComponentImpl};
use conductor::conductor::Conductor;
use scp::{
    AcpClientToAgentCallbacks, InitializeResponseExt, JsonRpcConnection, JsonRpcConnectionCx,
    JsonRpcCxExt, JsonRpcRequestCx, Proxy,
};
use std::sync::Arc;
use tokio::io::duplex;
use tokio::sync::Mutex;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::Instrument;

/// Test helper to receive a JSON-RPC response
async fn recv<R: scp::JsonRpcIncomingMessage + Send>(
    response: scp::JsonRpcResponse<R>,
) -> Result<R, agent_client_protocol::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    response.await_when_response_received(async move |result| {
        tx.send(result)
            .map_err(|_| agent_client_protocol::Error::internal_error())
    })?;
    rx.await
        .map_err(|_| agent_client_protocol::Error::internal_error())?
}

/// Mock component that captures what InitializeRequest it receives
fn mock_component_that_captures_init(
    accept_proxy: bool,
) -> (MockComponentImpl, Arc<Mutex<Option<InitializeRequest>>>) {
    let captured = Arc::new(Mutex::new(None));

    let mock = MockComponentImpl::new({
        let captured = captured.clone();
        async move |connection| {
            let _ = connection
                .on_receive(scp::AcpClientToAgentMessages::callback(
                    CapturingCallbacks {
                        captured_init: captured,
                        accept_proxy,
                    },
                ))
                .serve()
                .instrument(tracing::info_span!("actor", id = "mock_component"))
                .await;
        }
    });

    (mock, captured)
}

/// Callbacks that capture initialization and optionally accept proxy capability
struct CapturingCallbacks {
    captured_init: Arc<Mutex<Option<InitializeRequest>>>,
    accept_proxy: bool,
}

impl AcpClientToAgentCallbacks for CapturingCallbacks {
    async fn initialize(
        &mut self,
        args: InitializeRequest,
        response: JsonRpcRequestCx<InitializeResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        // Capture what we received
        let has_proxy_offer = args
            .meta
            .as_ref()
            .and_then(|m| m.get("symposium"))
            .and_then(|s| s.get("proxy"))
            .and_then(|p| p.as_bool())
            .unwrap_or(false);

        *self.captured_init.lock().await = Some(args.clone());

        // If we were offered proxy capability, we need to act as a proxy and forward
        if has_proxy_offer {
            if self.accept_proxy {
                // Forward to successor, get response, add proxy capability, and send back
                let successor_response = response.send_request_to_successor(args);

                let _ = successor_response.await_when_response_received(async move |result| {
                    match result {
                        Ok(mut init_response) => {
                            // Add proxy capability to our response
                            init_response = init_response.add_meta_capability(Proxy);
                            response.respond(init_response)
                        }
                        Err(e) => response.respond_with_error(e),
                    }
                });
            } else {
                // We were offered proxy but we're refusing - just respond without the capability
                let init_response = InitializeResponse {
                    protocol_version: Default::default(),
                    agent_capabilities: Default::default(),
                    auth_methods: vec![],
                    meta: None,
                };
                let _ = response.respond(init_response);
            }
        } else {
            // We're the agent (no proxy offer), just respond normally
            let init_response = InitializeResponse {
                protocol_version: Default::default(),
                agent_capabilities: Default::default(),
                auth_methods: vec![],
                meta: None,
            };
            let _ = response.respond(init_response);
        }

        Ok(())
    }

    async fn authenticate(
        &mut self,
        _args: agent_client_protocol::AuthenticateRequest,
        _response: JsonRpcRequestCx<agent_client_protocol::AuthenticateResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn session_cancel(
        &mut self,
        _args: agent_client_protocol::CancelNotification,
        _cx: &JsonRpcConnectionCx,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn new_session(
        &mut self,
        _args: agent_client_protocol::NewSessionRequest,
        _response: JsonRpcRequestCx<agent_client_protocol::NewSessionResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn load_session(
        &mut self,
        _args: agent_client_protocol::LoadSessionRequest,
        _response: JsonRpcRequestCx<agent_client_protocol::LoadSessionResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn prompt(
        &mut self,
        _args: agent_client_protocol::PromptRequest,
        _response: JsonRpcRequestCx<agent_client_protocol::PromptResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn set_session_mode(
        &mut self,
        _args: agent_client_protocol::SetSessionModeRequest,
        _response: JsonRpcRequestCx<agent_client_protocol::SetSessionModeResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn test_single_component_no_proxy_offer() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("conductor=debug".parse().unwrap()),
        )
        .with_test_writer()
        .try_init();

    let local = tokio::task::LocalSet::new();

    local
        .run_until(async {
            // Create a single mock component
            let (mock, captured_init) = mock_component_that_captures_init(false);

            // Set up editor <-> conductor communication
            let (editor_out, conductor_in) = duplex(1024);
            let (conductor_out, editor_in) = duplex(1024);

            // Spawn conductor with single component
            let conductor_handle = tokio::task::spawn_local(async move {
                Conductor::run(
                    conductor_out.compat_write(),
                    conductor_in.compat(),
                    vec![ComponentProvider::Mock(Box::new(mock))],
                )
                .await
            });

            // Editor sends initialize
            let editor_task = tokio::task::spawn_local(async move {
                JsonRpcConnection::new(editor_out.compat_write(), editor_in.compat())
                    .with_client(async move |client| {
                        let init_response = recv(client.send_request(InitializeRequest {
                            protocol_version: Default::default(),
                            client_capabilities: Default::default(),
                            meta: None,
                        }))
                        .await;

                        assert!(
                            init_response.is_ok(),
                            "Initialize should succeed: {:?}",
                            init_response
                        );

                        Ok::<(), agent_client_protocol::Error>(())
                    })
                    .await
            });

            let _ = editor_task.await.expect("Editor task completes");

            // Verify the component did NOT receive proxy capability offer
            let received = captured_init.lock().await;
            assert!(received.is_some(), "Component should receive initialize");

            let has_proxy_offer = received
                .as_ref()
                .unwrap()
                .meta
                .as_ref()
                .and_then(|m| m.get("symposium"))
                .and_then(|s| s.get("proxy"))
                .and_then(|p| p.as_bool())
                .unwrap_or(false);

            assert!(
                !has_proxy_offer,
                "Single component should NOT be offered proxy capability"
            );

            conductor_handle.abort();
        })
        .await;
}

#[tokio::test]
async fn test_two_components_proxy_offered_and_accepted() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("conductor=debug".parse().unwrap()),
        )
        .with_test_writer()
        .try_init();

    let local = tokio::task::LocalSet::new();

    local
        .run_until(async {
            // Component 1: proxy (should be offered and should accept)
            let (proxy_mock, proxy_captured) = mock_component_that_captures_init(true);

            // Component 2: agent (should NOT be offered proxy)
            let (agent_mock, agent_captured) = mock_component_that_captures_init(false);

            let (editor_out, conductor_in) = duplex(1024);
            let (conductor_out, editor_in) = duplex(1024);

            let conductor_handle = tokio::task::spawn_local(async move {
                Conductor::run(
                    conductor_out.compat_write(),
                    conductor_in.compat(),
                    vec![
                        ComponentProvider::Mock(Box::new(proxy_mock)),
                        ComponentProvider::Mock(Box::new(agent_mock)),
                    ],
                )
                .await
            });

            let editor_task = tokio::task::spawn_local(async move {
                JsonRpcConnection::new(editor_out.compat_write(), editor_in.compat())
                    .with_client(async move |client| {
                        let init_response = recv(client.send_request(InitializeRequest {
                            protocol_version: Default::default(),
                            client_capabilities: Default::default(),
                            meta: None,
                        }))
                        .await;

                        assert!(
                            init_response.is_ok(),
                            "Initialize should succeed: {:?}",
                            init_response
                        );

                        Ok::<(), agent_client_protocol::Error>(())
                    })
                    .await
            });

            let _ = editor_task.await.expect("Editor task completes");

            // Verify proxy component received offer
            let proxy_init = proxy_captured.lock().await;
            assert!(proxy_init.is_some(), "Proxy should receive initialize");

            let proxy_offered = proxy_init
                .as_ref()
                .unwrap()
                .meta
                .as_ref()
                .and_then(|m| m.get("symposium"))
                .and_then(|s| s.get("proxy"))
                .and_then(|p| p.as_bool())
                .unwrap_or(false);

            assert!(
                proxy_offered,
                "First component (proxy) should be offered proxy capability"
            );

            // Verify agent component did NOT receive offer
            let agent_init = agent_captured.lock().await;
            assert!(agent_init.is_some(), "Agent should receive initialize");

            let agent_offered = agent_init
                .as_ref()
                .unwrap()
                .meta
                .as_ref()
                .and_then(|m| m.get("symposium"))
                .and_then(|s| s.get("proxy"))
                .and_then(|p| p.as_bool())
                .unwrap_or(false);

            assert!(
                !agent_offered,
                "Last component (agent) should NOT be offered proxy capability"
            );

            conductor_handle.abort();
        })
        .await;
}

#[tokio::test]
async fn test_proxy_handshake_failure() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("conductor=debug".parse().unwrap()),
        )
        .with_test_writer()
        .try_init();

    let local = tokio::task::LocalSet::new();

    local
        .run_until(async {
            // Component 1: proxy that REFUSES to accept proxy capability (accept_proxy = false)
            let (proxy_mock, _) = mock_component_that_captures_init(false);

            // Component 2: agent
            let (agent_mock, _) = mock_component_that_captures_init(false);

            let (editor_out, conductor_in) = duplex(1024);
            let (conductor_out, editor_in) = duplex(1024);

            let conductor_handle = tokio::task::spawn_local(async move {
                Conductor::run(
                    conductor_out.compat_write(),
                    conductor_in.compat(),
                    vec![
                        ComponentProvider::Mock(Box::new(proxy_mock)),
                        ComponentProvider::Mock(Box::new(agent_mock)),
                    ],
                )
                .await
            });

            let editor_task = tokio::task::spawn_local(async move {
                JsonRpcConnection::new(editor_out.compat_write(), editor_in.compat())
                    .with_client(async move |client| {
                        let init_response = recv(client.send_request(InitializeRequest {
                            protocol_version: Default::default(),
                            client_capabilities: Default::default(),
                            meta: None,
                        }))
                        .await;

                        // Should fail because proxy didn't accept the capability
                        assert!(
                            init_response.is_err(),
                            "Initialize should fail when proxy doesn't accept capability"
                        );

                        let err = init_response.unwrap_err();
                        let err_data = format!("{:?}", err);
                        assert!(
                            err_data.contains("is not a proxy") || err_data.contains("component 0"),
                            "Error should mention component is not a proxy: {}",
                            err_data
                        );

                        Ok::<(), agent_client_protocol::Error>(())
                    })
                    .await
            });

            let _ = editor_task.await.expect("Editor task completes");

            conductor_handle.abort();
        })
        .await;
}
