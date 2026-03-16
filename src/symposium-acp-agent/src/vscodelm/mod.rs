//! VS Code Language Model Provider backend
//!
//! This module implements the Rust backend for the VS Code `LanguageModelChatProvider` API.
//! It uses sacp's JSON-RPC infrastructure for communication with the TypeScript extension.

mod history_actor;
pub mod session_actor;
#[cfg(test)]
mod tests;
mod vscode_tools_mcp;

use anyhow::Result;
use history_actor::{HistoryActor, HistoryActorHandle};
use sacp::{
    ConnectTo, Dispatch, HandleDispatchFrom, JsonRpcNotification, JsonRpcRequest, Role,
    util::MatchDispatch,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Name of the special tool we inject into vscode for requesting permission
const SYMPOSIUM_AGENT_ACTION: &str = "symposium-agent-action";

/// Role constants for message matching
pub const ROLE_USER: &str = "user";
pub const ROLE_ASSISTANT: &str = "assistant";

// ============================================================================
// Peers
// ============================================================================

/// Peer representing the VS Code extension (TypeScript side).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VsCode;

/// Peer representing the LM backend (Rust side).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LmBackend;

impl sacp::Role for VsCode {
    type Counterpart = LmBackend;

    fn role_id(&self) -> sacp::role::RoleId {
        sacp::role::RoleId::from_singleton(self)
    }

    fn default_handle_dispatch_from(
        &self,
        message: sacp::Dispatch,
        _connection: sacp::ConnectionTo<Self>,
    ) -> impl std::future::Future<Output = Result<sacp::Handled<sacp::Dispatch>, sacp::Error>> + Send
    {
        async move {
            Ok(sacp::Handled::No {
                message,
                retry: false,
            })
        }
    }

    fn counterpart(&self) -> Self::Counterpart {
        LmBackend
    }
}

impl sacp::Role for LmBackend {
    type Counterpart = VsCode;

    fn role_id(&self) -> sacp::role::RoleId {
        sacp::role::RoleId::from_singleton(self)
    }

    fn default_handle_dispatch_from(
        &self,
        message: sacp::Dispatch,
        _connection: sacp::ConnectionTo<Self>,
    ) -> impl std::future::Future<Output = Result<sacp::Handled<sacp::Dispatch>, sacp::Error>> + Send
    {
        async move {
            Ok(sacp::Handled::No {
                message,
                retry: false,
            })
        }
    }

    fn counterpart(&self) -> Self::Counterpart {
        VsCode
    }
}

impl sacp::role::HasPeer<VsCode> for LmBackend {
    fn remote_style(&self, _peer: VsCode) -> sacp::role::RemoteStyle {
        sacp::role::RemoteStyle::Counterpart
    }
}

impl sacp::role::HasPeer<LmBackend> for VsCode {
    fn remote_style(&self, _peer: LmBackend) -> sacp::role::RemoteStyle {
        sacp::role::RemoteStyle::Counterpart
    }
}

impl ConnectTo<LmBackend> for VsCode {
    async fn connect_to(self, client: impl ConnectTo<VsCode>) -> Result<(), sacp::Error> {
        VsCode::builder(self).connect_to(client).await
    }
}

impl ConnectTo<VsCode> for LmBackend {
    async fn connect_to(self, client: impl ConnectTo<LmBackend>) -> Result<(), sacp::Error> {
        LmBackend::builder(self)
            .with_handler(LmBackendHandler::new())
            .connect_to(client)
            .await
    }
}

// ============================================================================
// Message Types
// ============================================================================

/// Message content part
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        value: String,
    },
    ToolCall {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(rename = "toolName")]
        tool_name: String,
        parameters: serde_json::Value,
    },
    ToolResult {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        result: serde_json::Value,
    },
}

/// A chat message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentPart>,
}

impl Message {
    /// Extract text content from the message
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|part| match part {
                ContentPart::Text { value } => Some(value.as_str()),
                ContentPart::ToolCall { .. } | ContentPart::ToolResult { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the message contains a tool result for the given tool call ID
    pub fn has_tool_result(&self, tool_call_id: &str) -> bool {
        self.content.iter().any(|part| {
            matches!(part, ContentPart::ToolResult { tool_call_id: id, .. } if id == tool_call_id)
        })
    }

    /// Check if the message contains ONLY a tool result for the given tool call ID and nothing else
    pub fn has_just_tool_result(&self, tool_call_id: &str) -> bool {
        self.content.len() == 1 && self.has_tool_result(tool_call_id)
    }

    /// Normalize the message by coalescing consecutive Text parts.
    pub fn normalize(&mut self) {
        let mut normalized = Vec::with_capacity(self.content.len());
        for part in self.content.drain(..) {
            if let ContentPart::Text { value: new_text } = &part {
                if let Some(ContentPart::Text { value: existing }) = normalized.last_mut() {
                    existing.push_str(new_text);
                    continue;
                }
            }
            normalized.push(part);
        }
        self.content = normalized;
    }
}

/// Normalize a vector of messages in place.
pub fn normalize_messages(messages: &mut Vec<Message>) {
    for msg in messages.iter_mut() {
        msg.normalize();
    }
}

// ============================================================================
// Request Options Types
// ============================================================================

/// Tool definition passed in request options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool mode for chat requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolMode {
    #[default]
    Auto,
    Required,
}

/// Options for chat requests from VS Code
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequestOptions {
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    #[serde(default)]
    pub tool_mode: Option<ToolMode>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub family: String,
    pub version: String,
    pub max_input_tokens: u32,
    pub max_output_tokens: u32,
    pub capabilities: ModelCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    #[serde(default)]
    pub tool_calling: bool,
}

// ----------------------------------------------------------------------------
// lm/provideLanguageModelChatInformation
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcRequest)]
#[request(method = "lm/provideLanguageModelChatInformation", response = ProvideInfoResponse)]
pub struct ProvideInfoRequest {
    #[serde(default)]
    pub silent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvideInfoResponse {
    pub models: Vec<ModelInfo>,
}

impl sacp::JsonRpcResponse for ProvideInfoResponse {
    fn into_json(self, _version: &str) -> Result<serde_json::Value, sacp::Error> {
        serde_json::to_value(self).map_err(Into::into)
    }

    fn from_value(_version: &str, v: serde_json::Value) -> Result<Self, sacp::Error> {
        serde_json::from_value(v).map_err(Into::into)
    }
}

// ----------------------------------------------------------------------------
// lm/provideLanguageModelChatResponse
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcRequest)]
#[request(method = "lm/provideLanguageModelChatResponse", response = ProvideResponseResponse)]
#[serde(rename_all = "camelCase")]
pub struct ProvideResponseRequest {
    pub model_id: String,
    pub messages: Vec<Message>,
    pub agent: session_actor::AgentDefinition,
    #[serde(default)]
    pub options: ChatRequestOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvideResponseResponse {}

impl sacp::JsonRpcResponse for ProvideResponseResponse {
    fn into_json(self, _version: &str) -> Result<serde_json::Value, sacp::Error> {
        serde_json::to_value(self).map_err(Into::into)
    }

    fn from_value(_version: &str, v: serde_json::Value) -> Result<Self, sacp::Error> {
        serde_json::from_value(v).map_err(Into::into)
    }
}

// ----------------------------------------------------------------------------
// lm/responsePart (notification: backend -> vscode)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcNotification)]
#[notification(method = "lm/responsePart")]
#[serde(rename_all = "camelCase")]
pub struct ResponsePartNotification {
    pub request_id: serde_json::Value,
    pub part: ContentPart,
}

// ----------------------------------------------------------------------------
// lm/responseComplete (notification: backend -> vscode)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcNotification)]
#[notification(method = "lm/responseComplete")]
#[serde(rename_all = "camelCase")]
pub struct ResponseCompleteNotification {
    pub request_id: serde_json::Value,
}

// ----------------------------------------------------------------------------
// lm/cancel (notification: vscode -> backend)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcNotification)]
#[notification(method = "lm/cancel")]
#[serde(rename_all = "camelCase")]
pub struct CancelNotification {
    pub request_id: serde_json::Value,
}

// ----------------------------------------------------------------------------
// lm/provideTokenCount
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonRpcRequest)]
#[request(method = "lm/provideTokenCount", response = ProvideTokenCountResponse)]
#[serde(rename_all = "camelCase")]
pub struct ProvideTokenCountRequest {
    pub model_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvideTokenCountResponse {
    pub count: u32,
}

impl sacp::JsonRpcResponse for ProvideTokenCountResponse {
    fn into_json(self, _version: &str) -> Result<serde_json::Value, sacp::Error> {
        serde_json::to_value(self).map_err(Into::into)
    }

    fn from_value(_version: &str, v: serde_json::Value) -> Result<Self, sacp::Error> {
        serde_json::from_value(v).map_err(Into::into)
    }
}

// ============================================================================
// Message Handler
// ============================================================================

/// Handler for LM backend messages.
/// Forwards requests to HistoryActor for actual processing.
pub struct LmBackendHandler {
    /// Handle to send messages to the HistoryActor.
    /// Created lazily on first request that needs it.
    history_handle: Option<HistoryActorHandle>,
}

impl LmBackendHandler {
    pub fn new() -> Self {
        Self {
            history_handle: None,
        }
    }

    /// Get or create the history actor handle.
    /// The actor is created lazily on first use, using the provided connection context.
    fn get_or_create_history_handle(
        &mut self,
        cx: &sacp::ConnectionTo<VsCode>,
    ) -> Result<&HistoryActorHandle, sacp::Error> {
        if self.history_handle.is_none() {
            let handle = HistoryActor::new(&cx)?;
            self.history_handle = Some(handle);
        }
        Ok(self.history_handle.as_ref().unwrap())
    }
}

impl HandleDispatchFrom<VsCode> for LmBackendHandler {
    fn describe_chain(&self) -> impl std::fmt::Debug {
        "LmBackendHandler"
    }

    async fn handle_dispatch_from(
        &mut self,
        message: sacp::Dispatch,
        connection: sacp::ConnectionTo<VsCode>,
    ) -> std::result::Result<sacp::Handled<sacp::Dispatch>, sacp::Error> {
        tracing::trace!(?message, "handle_message");

        // Get or create the history actor handle (lazy init on first call)
        let history_handle = self.get_or_create_history_handle(&connection)?.clone();

        MatchDispatch::new(message)
            .if_request(async |_req: ProvideInfoRequest, request_cx| {
                let response = ProvideInfoResponse {
                    models: vec![ModelInfo {
                        id: "symposium-eliza".to_string(),
                        name: "Symposium (Eliza)".to_string(),
                        family: "symposium".to_string(),
                        version: "1.0.0".to_string(),
                        max_input_tokens: 100000,
                        max_output_tokens: 100000,
                        capabilities: ModelCapabilities { tool_calling: true },
                    }],
                };
                request_cx.respond(response)
            })
            .await
            .if_request(async |req: ProvideTokenCountRequest, request_cx| {
                // Simple heuristic: 1 token ≈ 4 characters
                let count = (req.text.len() / 4).max(1) as u32;
                request_cx.respond(ProvideTokenCountResponse { count })
            })
            .await
            .if_request(async |req: ProvideResponseRequest, request_cx| {
                tracing::debug!(?req, "ProvideResponseRequest");

                let request_id = request_cx.id().clone();

                // Forward to HistoryActor for processing
                history_handle.send_from_vscode(req, request_id, request_cx)?;

                Ok(())
            })
            .await
            .if_notification(async |notification: CancelNotification| {
                tracing::debug!(?notification, "CancelNotification");

                // Forward to HistoryActor
                history_handle.send_cancel_from_vscode(notification.request_id)?;

                Ok(())
            })
            .await
            .otherwise(async |message| match message {
                Dispatch::Request(request, request_cx) => {
                    tracing::warn!("unknown request method: {}", request.method());
                    request_cx.respond_with_error(sacp::Error::method_not_found())
                }
                Dispatch::Notification(notif) => {
                    tracing::warn!("unexpected notification: {}", notif.method());
                    Ok(())
                }
                Dispatch::Response(response, router) => router.respond_with_result(response),
            })
            .await?;

        Ok(sacp::Handled::Yes)
    }
}

// ============================================================================
// Server (for CLI usage)
// ============================================================================

/// Run the LM backend on stdio
pub async fn serve_stdio(trace_dir: Option<PathBuf>) -> Result<()> {
    let stdio = if let Some(dir) = trace_dir {
        std::fs::create_dir_all(&dir)?;
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let trace_path = dir.join(format!("vscodelm-{}.log", timestamp));
        let file = std::sync::Arc::new(std::sync::Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&trace_path)?,
        ));
        tracing::info!(?trace_path, "Logging vscodelm messages");

        sacp_tokio::Stdio::new().with_debug(move |line, direction| {
            use std::io::Write;
            let dir_str = match direction {
                sacp_tokio::LineDirection::Stdin => "recv",
                sacp_tokio::LineDirection::Stdout => "send",
                sacp_tokio::LineDirection::Stderr => "stderr",
            };
            if let Ok(mut f) = file.lock() {
                let _ = writeln!(
                    f,
                    "[{}] {}: {}",
                    chrono::Utc::now().to_rfc3339(),
                    dir_str,
                    line
                );
                let _ = f.flush();
            }
        })
    } else {
        sacp_tokio::Stdio::new()
    };

    LmBackend.builder().connect_to(stdio).await?;
    Ok(())
}
