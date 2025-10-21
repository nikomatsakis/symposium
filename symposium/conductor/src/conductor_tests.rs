use std::{future::Future, pin::Pin};

use agent_client_protocol::{InitializeRequest, InitializeResponse};
use futures::{AsyncRead, AsyncWrite};
use scp::{AcpClientToAgentMessages, JsonRpcConnection, JsonRpcCx};
use tokio::io::duplex;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::component::MockComponent;

/// A mock component that implements ACP protocol for testing.
///
/// Spawns a local task with a JsonRpcConnection to handle the component side.
struct PassthroughMockComponent;

impl MockComponent for PassthroughMockComponent {
    fn create(
        &self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = anyhow::Result<(
                        Pin<Box<dyn AsyncWrite + Send>>,
                        Pin<Box<dyn AsyncRead + Send>>,
                    )>,
                > + Send,
        >,
    > {
        Box::pin(async move {
            // Create two duplex pairs for bidirectional communication
            let (conductor_out, component_in) = duplex(1024);
            let (component_out, conductor_in) = duplex(1024);

            // Spawn local task to run the mock component's JSON-RPC handler
            tokio::task::spawn_local(async move {
                let _ = JsonRpcConnection::new(component_out.compat_write(), component_in.compat())
                    .on_receive(AcpClientToAgentMessages::callback(PassthroughCallbacks))
                    .serve()
                    .await;
            });

            // Return conductor's ends of the streams
            Ok((
                Box::pin(conductor_out.compat_write()) as Pin<Box<dyn AsyncWrite + Send>>,
                Box::pin(conductor_in.compat()) as Pin<Box<dyn AsyncRead + Send>>,
            ))
        })
    }
}

/// Simple callbacks that respond to initialize requests with minimal responses
struct PassthroughCallbacks;

impl scp::AcpClientToAgentCallbacks for PassthroughCallbacks {
    async fn initialize(
        &mut self,
        _args: InitializeRequest,
        response: scp::JsonRpcRequestCx<InitializeResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        let _ = response.respond(InitializeResponse {
            protocol_version: Default::default(),
            agent_capabilities: Default::default(),
            auth_methods: vec![],
            meta: None,
        });
        Ok(())
    }

    async fn authenticate(
        &mut self,
        _args: agent_client_protocol::AuthenticateRequest,
        _response: scp::JsonRpcRequestCx<agent_client_protocol::AuthenticateResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn session_cancel(
        &mut self,
        _args: agent_client_protocol::CancelNotification,
        _cx: &JsonRpcCx,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn new_session(
        &mut self,
        _args: agent_client_protocol::NewSessionRequest,
        _response: scp::JsonRpcRequestCx<agent_client_protocol::NewSessionResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn load_session(
        &mut self,
        _args: agent_client_protocol::LoadSessionRequest,
        _response: scp::JsonRpcRequestCx<agent_client_protocol::LoadSessionResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn prompt(
        &mut self,
        _args: agent_client_protocol::PromptRequest,
        _response: scp::JsonRpcRequestCx<agent_client_protocol::PromptResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn set_session_mode(
        &mut self,
        _args: agent_client_protocol::SetSessionModeRequest,
        _response: scp::JsonRpcRequestCx<agent_client_protocol::SetSessionModeResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_component_no_proxy_capability() {
        // TODO: Test that a single component chain doesn't add proxy capability
    }

    #[tokio::test]
    async fn test_two_component_chain_capabilities() {
        // TODO: Test that first component gets proxy capability, last doesn't
    }
}
