//! Mock proxy component that provides go_go_gadget_shoes MCP tool

use std::sync::Arc;

use agent_client_protocol::{
    InitializeRequest, InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse,
};
use conductor::component::MockComponentImpl;
use scp::{AcpClientToAgentCallbacks, AcpClientToAgentMessages, JsonRpcCx, JsonRpcRequestCx};
use tokio::sync::Mutex;
use tracing::Instrument;

/// State shared between mock proxy callbacks
#[derive(Clone)]
struct ProxyState {
    /// UUID for the MCP server this proxy provides
    mcp_server_uuid: String,
    /// Counter for tool invocations (for test assertions)
    tool_invocation_count: Arc<Mutex<u32>>,
}

/// Callbacks for the mock proxy component
struct ProxyCallbacks {
    state: ProxyState,
}

impl AcpClientToAgentCallbacks for ProxyCallbacks {
    async fn initialize(
        &mut self,
        args: InitializeRequest,
        response: JsonRpcRequestCx<InitializeResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        tracing::info!("Proxy: received initialize");

        // Check if we have the proxy capability (should be present)
        let has_proxy_capability = args
            .meta
            .as_ref()
            .and_then(|m| m.get("symposium"))
            .and_then(|s| s.get("proxy"))
            .and_then(|p| p.as_bool())
            .unwrap_or(false);

        tracing::info!("Proxy: has_proxy_capability = {}", has_proxy_capability);

        // TODO: If we have proxy capability, we need to initialize our successor
        // For now, just respond
        let _ = response.respond(InitializeResponse {
            protocol_version: Default::default(),
            agent_capabilities: Default::default(),
            auth_methods: vec![],
            meta: Some(serde_json::json!({
                "mcp_acp_transport": true
            })),
        });

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
        _cx: &JsonRpcCx,
    ) -> Result<(), agent_client_protocol::Error> {
        Ok(())
    }

    async fn new_session(
        &mut self,
        args: NewSessionRequest,
        response: JsonRpcRequestCx<NewSessionResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        tracing::info!("Proxy: received new_session");

        // TODO: Inject our MCP server into the tool list
        // TODO: Forward this to the successor using _proxy/successor/request

        // Build modified MCP servers with our server
        let _mcp_servers = args.mcp_servers;

        // TODO: Parse and add our MCP server with ACP transport
        // This requires constructing an McpServer struct properly

        // TODO: Send this via _proxy/successor/request to the agent
        // For now, just respond with dummy session ID
        let _ = response.respond(NewSessionResponse {
            session_id: "test-session-123".to_string().into(),
            modes: Default::default(),
            meta: None,
        });

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
        _args: PromptRequest,
        response: JsonRpcRequestCx<PromptResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        tracing::info!("Proxy: received prompt");

        // TODO: Forward to successor
        let _ = response.respond(PromptResponse {
            stop_reason: agent_client_protocol::StopReason::EndTurn,
            meta: None,
        });

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

/// Create a mock proxy component that provides the go_go_gadget_shoes tool
pub fn create_mock_proxy() -> MockComponentImpl {
    let state = ProxyState {
        mcp_server_uuid: uuid::Uuid::new_v4().to_string(),
        tool_invocation_count: Arc::new(Mutex::new(0)),
    };

    MockComponentImpl::new(move |connection| async move {
        let callbacks = ProxyCallbacks {
            state: state.clone(),
        };

        let _ = connection
            .on_receive(AcpClientToAgentMessages::callback(callbacks))
            .serve()
            .instrument(tracing::info_span!("actor", id = "mock_proxy"))
            .await;
    })
}
