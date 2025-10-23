//! Mock agent component that uses rmcp to invoke MCP tools

use agent_client_protocol::{
    InitializeRequest, InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse,
};
use conductor::component::MockComponentImpl;
use scp::{AcpClientToAgentCallbacks, AcpClientToAgentMessages, JsonRpcCx, JsonRpcRequestCx};
use tracing::Instrument;

/// Callbacks for the mock agent component
struct AgentCallbacks;

impl AcpClientToAgentCallbacks for AgentCallbacks {
    async fn initialize(
        &mut self,
        args: InitializeRequest,
        response: JsonRpcRequestCx<InitializeResponse>,
    ) -> Result<(), agent_client_protocol::Error> {
        tracing::info!("Agent: received initialize");

        // Agent should NOT receive proxy capability (it's the last component)
        let has_proxy_capability = args
            .meta
            .as_ref()
            .and_then(|m| m.get("symposium"))
            .and_then(|s| s.get("proxy"))
            .and_then(|p| p.as_bool())
            .unwrap_or(false);

        assert!(
            !has_proxy_capability,
            "Agent should not receive proxy capability"
        );

        // Agent does NOT support mcp_acp_transport
        let _ = response.respond(InitializeResponse {
            protocol_version: Default::default(),
            agent_capabilities: Default::default(),
            auth_methods: vec![],
            meta: None, // No mcp_acp_transport capability
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
        tracing::info!("Agent: received new_session");
        tracing::info!("Agent: MCP servers = {:?}", args.mcp_servers);

        // Agent should receive modified MCP server list with stdio transport
        // pointing to "conductor mcp $PORT"

        // TODO: Extract MCP server info
        // TODO: Use rmcp to connect to the server
        // TODO: Store rmcp connection for later use in prompt

        let _ = response.respond(NewSessionResponse {
            session_id: "agent-session-456".to_string().into(),
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
        tracing::info!("Agent: received prompt");

        // TODO: Use rmcp to invoke go_go_gadget_shoes tool
        // TODO: Wait for response
        // TODO: Send back response

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

/// Create a mock agent component that uses rmcp to invoke MCP tools
pub fn create_mock_agent() -> MockComponentImpl {
    MockComponentImpl::new(move |connection| async move {
        let callbacks = AgentCallbacks;

        let _ = connection
            .on_receive(AcpClientToAgentMessages::callback(callbacks))
            .serve()
            .instrument(tracing::info_span!("actor", id = "mock_agent"))
            .await;
    })
}
