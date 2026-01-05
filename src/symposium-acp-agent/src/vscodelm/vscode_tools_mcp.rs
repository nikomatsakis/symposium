//! Synthetic MCP server that exposes VS Code-provided tools to ACP agents.
//!
//! This module bridges VS Code's Language Model API tools to ACP agents by creating
//! an MCP server that:
//! 1. Advertises VS Code tools to the agent via `tools/list`
//! 2. Routes tool invocations back to VS Code via the session actor
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        Session Actor                                 │
//! │                                                                      │
//! │  ┌──────────────┐     tools_tx      ┌─────────────────────────────┐ │
//! │  │              │ ───────────────►  │                             │ │
//! │  │  Request     │                   │  VscodeToolsMcpServer       │ │
//! │  │  Handler     │  ◄───────────────  │  (rmcp ServerHandler)       │ │
//! │  │              │    invocation_rx  │                             │ │
//! │  └──────────────┘                   └─────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use std::borrow::Cow;
use std::sync::Arc;

use futures::channel::{mpsc, oneshot};
use rmcp::model::{
    CallToolRequestParam, CallToolResult, ErrorCode, ListToolsResult, PaginatedRequestParam,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use tokio::sync::RwLock;

/// A tool definition received from VS Code.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// A tool invocation request sent to the session actor.
#[derive(Debug)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
    pub result_tx: oneshot::Sender<Result<CallToolResult, String>>,
}

/// Shared state for the MCP server.
struct VscodeToolsState {
    /// Current list of tools from VS Code
    tools: Vec<VscodeTool>,
}

/// Synthetic MCP server that exposes VS Code tools to ACP agents.
#[derive(Clone)]
pub struct VscodeToolsMcpServer {
    state: Arc<RwLock<VscodeToolsState>>,
    invocation_tx: mpsc::UnboundedSender<ToolInvocation>,
}

impl VscodeToolsMcpServer {
    /// Create a new VS Code tools MCP server.
    ///
    /// Takes a sender for tool invocations that will be used when the agent calls a tool.
    pub fn new(invocation_tx: mpsc::UnboundedSender<ToolInvocation>) -> Self {
        Self {
            state: Arc::new(RwLock::new(VscodeToolsState { tools: Vec::new() })),
            invocation_tx,
        }
    }

    /// Get a handle that can be used to update tools from another task.
    pub fn tools_handle(&self) -> VscodeToolsHandle {
        VscodeToolsHandle {
            state: self.state.clone(),
        }
    }
}

/// Handle for updating tools from outside the MCP server.
#[derive(Clone)]
pub struct VscodeToolsHandle {
    state: Arc<RwLock<VscodeToolsState>>,
}

impl VscodeToolsHandle {
    /// Update the list of available tools.
    pub async fn update_tools(&self, tools: Vec<VscodeTool>) {
        let mut state = self.state.write().await;
        state.tools = tools;
    }
}

impl ServerHandler for VscodeToolsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
            server_info: rmcp::model::Implementation {
                name: "symposium-vscode-tools".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some("VS Code-provided tools bridged to ACP".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let state = self.state.read().await;

        let tools: Vec<Tool> = state
            .tools
            .iter()
            .map(|t| {
                let input_schema = match &t.input_schema {
                    serde_json::Value::Object(obj) => Arc::new(obj.clone()),
                    _ => Arc::new(serde_json::Map::new()),
                };
                Tool {
                    name: Cow::Owned(t.name.clone()),
                    title: None,
                    description: Some(Cow::Owned(t.description.clone())),
                    input_schema,
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: None,
                }
            })
            .collect();

        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        // Check if tool exists
        {
            let state = self.state.read().await;
            if !state.tools.iter().any(|t| t.name == request.name.as_ref()) {
                return Err(ErrorData::new(
                    ErrorCode::INVALID_PARAMS,
                    format!("tool '{}' not found", request.name),
                    None,
                ));
            }
        }

        // Create a oneshot channel for the result
        let (result_tx, result_rx) = oneshot::channel();

        // Send invocation to session actor
        let invocation = ToolInvocation {
            name: request.name.to_string(),
            arguments: request.arguments,
            result_tx,
        };

        self.invocation_tx
            .unbounded_send(invocation)
            .map_err(|_| ErrorData::internal_error("session actor unavailable", None))?;

        // Wait for result from session actor
        match result_rx.await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(error)) => Err(ErrorData::internal_error(error, None)),
            Err(_) => Err(ErrorData::internal_error("tool invocation cancelled", None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_and_list_tools() {
        let (invocation_tx, _invocation_rx) = mpsc::unbounded();
        let server = VscodeToolsMcpServer::new(invocation_tx);
        let handle = server.tools_handle();

        // Initially empty - check via internal state
        {
            let state = server.state.read().await;
            assert!(state.tools.is_empty());
        }

        // Update tools via handle
        handle
            .update_tools(vec![VscodeTool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
            }])
            .await;

        // Now has one tool
        {
            let state = server.state.read().await;
            assert_eq!(state.tools.len(), 1);
            assert_eq!(state.tools[0].name, "test_tool");
        }
    }
}
