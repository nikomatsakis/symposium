//! VS Code Language Model Provider backend
//!
//! This module implements the Rust backend for the VS Code `LanguageModelChatProvider` API.
//! It uses sacp's JSON-RPC infrastructure for communication with the TypeScript extension.

mod session_actor;

use anyhow::Result;
use sacp::{
    link::RemoteStyle, util::MatchMessage, Component, Handled, JrConnectionCx, JrLink,
    JrMessageHandler, JrNotification, JrPeer, JrRequest, JrRequestCx, JrResponsePayload, MessageCx,
};
use serde::{Deserialize, Serialize};
use session_actor::SessionActor;
use std::{path::PathBuf, pin::pin};

/// Name of the special tool we inject into vscode for requesting permission
const SYMPOSIUM_AGENT_ACTION: &str = "symposium-agent-action";

// ============================================================================
// Peers
// ============================================================================

/// Peer representing the VS Code extension (TypeScript side).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VsCodePeer;

impl JrPeer for VsCodePeer {}

/// Peer representing the LM backend (Rust side).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LmBackendPeer;

impl JrPeer for LmBackendPeer {}

// ============================================================================
// Links
// ============================================================================

/// Link from the LM backend's perspective (talking to VS Code).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LmBackendToVsCode;

impl JrLink for LmBackendToVsCode {
    type ConnectsTo = VsCodeToLmBackend;
    type State = ();
}

impl sacp::HasDefaultPeer for LmBackendToVsCode {
    type DefaultPeer = VsCodePeer;
}

impl sacp::HasPeer<VsCodePeer> for LmBackendToVsCode {
    fn remote_style(_peer: VsCodePeer) -> RemoteStyle {
        RemoteStyle::Counterpart
    }
}

/// Link from VS Code's perspective (talking to the LM backend).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VsCodeToLmBackend;

impl JrLink for VsCodeToLmBackend {
    type ConnectsTo = LmBackendToVsCode;
    type State = ();
}

impl sacp::HasDefaultPeer for VsCodeToLmBackend {
    type DefaultPeer = LmBackendPeer;
}

impl sacp::HasPeer<LmBackendPeer> for VsCodeToLmBackend {
    fn remote_style(_peer: LmBackendPeer) -> RemoteStyle {
        RemoteStyle::Counterpart
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

    /// Check if the message contains a tool call with the given ID
    pub fn has_tool_call(&self, tool_call_id: &str) -> bool {
        self.content.iter().any(|part| {
            matches!(part, ContentPart::ToolCall { tool_call_id: id, .. } if id == tool_call_id)
        })
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, JrRequest)]
#[request(method = "lm/provideLanguageModelChatInformation", response = ProvideInfoResponse)]
pub struct ProvideInfoRequest {
    #[serde(default)]
    pub silent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JrResponsePayload)]
pub struct ProvideInfoResponse {
    pub models: Vec<ModelInfo>,
}

// ----------------------------------------------------------------------------
// lm/provideLanguageModelChatResponse
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JrRequest)]
#[request(method = "lm/provideLanguageModelChatResponse", response = ProvideResponseResponse)]
#[serde(rename_all = "camelCase")]
pub struct ProvideResponseRequest {
    pub model_id: String,
    pub messages: Vec<Message>,
    pub agent: session_actor::AgentDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JrResponsePayload)]
pub struct ProvideResponseResponse {}

// ----------------------------------------------------------------------------
// lm/responsePart (notification: backend -> vscode)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JrNotification)]
#[notification(method = "lm/responsePart")]
#[serde(rename_all = "camelCase")]
pub struct ResponsePartNotification {
    pub request_id: serde_json::Value,
    pub part: ContentPart,
}

// ----------------------------------------------------------------------------
// lm/responseComplete (notification: backend -> vscode)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JrNotification)]
#[notification(method = "lm/responseComplete")]
#[serde(rename_all = "camelCase")]
pub struct ResponseCompleteNotification {
    pub request_id: serde_json::Value,
}

// ----------------------------------------------------------------------------
// lm/cancel (notification: vscode -> backend)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JrNotification)]
#[notification(method = "lm/cancel")]
#[serde(rename_all = "camelCase")]
pub struct CancelNotification {
    pub request_id: serde_json::Value,
}

// ----------------------------------------------------------------------------
// lm/provideTokenCount
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JrRequest)]
#[request(method = "lm/provideTokenCount", response = ProvideTokenCountResponse)]
#[serde(rename_all = "camelCase")]
pub struct ProvideTokenCountRequest {
    pub model_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JrResponsePayload)]
pub struct ProvideTokenCountResponse {
    pub count: u32,
}

// ============================================================================
// Message Handler
// ============================================================================

use futures::{channel::oneshot, stream};

use crate::vscodelm::session_actor::{ActivePrompt, SessionToCodeMessage};

/// A session with its current state.
struct SessionData {
    actor: SessionActor,
    state: SessionState,
}

/// State of a session from the handler's perspective.
enum SessionState {
    /// Session is idle, waiting for a prompt.
    Idle,

    /// Session is streaming a response.
    Streaming {
        /// The JSON-RPC request ID of the in-flight request.
        request_id: serde_json::Value,
        /// Send on this channel to cancel the streaming response.
        cancel_tx: oneshot::Sender<()>,
    },

    /// Session is awaiting permission decision from VS Code.
    AwaitingPermission {
        /// The tool call ID we emitted to VS Code for the permission request.
        tool_call_id: String,

        /// Where the decision should be sent (send a `()` if approved, drop otherwise)
        decision_tx: oneshot::Sender<()>,
    },
}

impl SessionState {
    /// Cancel any in-progress streaming and transition to Idle.
    /// No-op if already Idle or AwaitingPermission.
    fn cancel(&mut self) {
        let old_state = std::mem::replace(self, SessionState::Idle);
        if let SessionState::Streaming { cancel_tx, .. } = old_state {
            // Ignore send error - receiver may already be gone
            let _ = cancel_tx.send(());
        }
        // AwaitingPermission doesn't need special cleanup - the actor handles it
    }
}

impl SessionData {
    /// Check if incoming messages extend this session's history.
    fn prefix_match_len(&self, messages: &[Message]) -> Option<usize> {
        self.actor.prefix_match_len(messages)
    }

    /// Returns true if this session is streaming with the given request ID.
    fn is_streaming_request(&self, request_id: &serde_json::Value) -> bool {
        matches!(&self.state, SessionState::Streaming { request_id: rid, .. } if rid == request_id)
    }

    /// Check if the session is awaiting permission and the messages contain approval.
    ///
    /// Returns Some(true) if approved (tool result present), Some(false) if rejected
    /// (no tool call/result in messages), None if not awaiting permission.
    fn check_permission_in_messages(&self, messages: &[Message]) -> Option<bool> {
        let tool_call_id = self.state.awaiting_permission_for()?;

        // Look for the tool call and result in messages
        // If we find both the tool call we emitted AND a corresponding result,
        // VS Code approved the action
        let has_tool_call = messages.iter().any(|m| m.has_tool_call(tool_call_id));
        let has_tool_result = messages.iter().any(|m| m.has_tool_result(tool_call_id));

        if has_tool_call && has_tool_result {
            Some(true) // Approved
        } else {
            Some(false) // Rejected (no matching tool call/result)
        }
    }

    fn receive_messages(
        &mut self,
        request_id: &serde_json::Value,
        new_messages: &[Message],
        request_cx: JrRequestCx<ProvideResponseResponse>,
        cx: &JrConnectionCx<LmBackendToVsCode>,
    ) -> Result<(), sacp::Error> {
        match self.state {
            SessionState::Idle => {
                self.receive_messages_when_idle(request_id, new_messages, request_cx, cx)
            }
            SessionState::Streaming {
                request_id,
                cancel_tx,
            } => todo!(),
            SessionState::AwaitingPermission {
                tool_call_id,
                decision_tx,
            } => todo!(),
        }
    }

    /// Receive messages when in the idle state.
    fn receive_messages_when_idle(
        &mut self,
        request_id: &serde_json::Value,
        new_messages: &[Message],
        request_cx: JrRequestCx<ProvideResponseResponse>,
        cx: &JrConnectionCx<LmBackendToVsCode>,
    ) -> Result<(), sacp::Error> {
        let state = std::mem::replace(&mut self.state, SessionState::Idle);
        assert!(matches!(state, SessionState::Idle));

        // Create cancellation channel
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Send prompt to actor
        let active_prompt = self.actor.send_prompt(new_messages.to_vec())?;

        // Transition to Streaming state
        self.state = SessionState::Streaming {
            request_id: request_id.clone(),
            cancel_tx,
        };

        // Spawn task to stream response (non-blocking)
        let cx = cx.clone();
        let request_id = request_id.clone();
        cx.clone().spawn(async move {
            stream_response(cx, request_id, request_cx, active_prompt, cancel_rx)
                .await
                .map(|_| ())
        })?;

        Ok(())
    }
}

/// Handler for LM backend messages
pub struct LmBackendHandler {
    /// Active sessions, searched linearly for prefix matches
    sessions: Vec<SessionData>,
}

impl LmBackendHandler {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }
}

/// JSON-RPC error code for request cancellation.
/// Using -32800 which is in the server error range (-32000 to -32099 reserved for implementation).
const ERROR_CODE_CANCELLED: i32 = -32800;

/// How a streaming response ended.
#[derive(Debug)]
pub enum StreamEndReason {
    /// Stream completed normally (agent finished its turn).
    Complete,
    /// Stream was cancelled by the client.
    Cancelled,
    /// Stream paused awaiting permission decision.
    AwaitingPermission { tool_call_id: String },
}

/// Stream response parts from the session actor, with cancellation support.
///
/// Merges the response part stream with a cancellation signal stream.
/// On normal completion, sends `lm/responseComplete` and responds to the request.
/// On cancellation, responds with a cancellation error.
/// On awaiting permission, responds successfully (VS Code will send another request).
///
/// Returns the reason the stream ended, which can be used to update session state.
async fn stream_response(
    cx: JrConnectionCx<LmBackendToVsCode>,
    request_id: serde_json::Value,
    request_cx: sacp::JrRequestCx<ProvideResponseResponse>,
    mut active_prompt: ActivePrompt,
    cancel_rx: oneshot::Receiver<()>,
) -> Result<StreamEndReason, sacp::Error> {
    use futures::StreamExt;
    use futures_concurrency::stream::Merge;

    enum Event {
        SessionToCode(SessionToCodeMessage),
        StreamEnded,
        Cancelled,
    }

    // Convert response stream to events, with StreamEnded when it closes
    let part_stream = active_prompt
        .prompt_rx()
        .by_ref()
        .map(Event::SessionToCode)
        .chain(stream::once(async { Event::StreamEnded }));

    // Convert cancel oneshot to a single-item stream
    let cancel_stream = stream::once(cancel_rx).map(|_| Event::Cancelled);

    // Merge both streams and pin for iteration
    let mut events = pin!((part_stream, cancel_stream).merge());

    while let Some(event) = events.next().await {
        match event {
            Event::SessionToCode(SessionToCodeMessage::Part(part)) => {
                cx.send_notification(ResponsePartNotification {
                    request_id: request_id.clone(),
                    part,
                })?;
            }
            Event::SessionToCode(SessionToCodeMessage::PermissionRequest {
                tool_call_id,
                kind,
                title,
                locations,
                raw_input,
                decision_tx,
            }) => {
                // Stream is pausing for permission - respond successfully
                // VS Code will send another request with the tool result if approved
                tracing::debug!(?request_id, %tool_call_id, "stream pausing for permission");
                cx.send_notification(ResponseCompleteNotification {
                    request_id: request_id.clone(),
                })?;
                request_cx.respond(ProvideResponseResponse {})?;
                return Ok(StreamEndReason::AwaitingPermission { tool_call_id });
            }
            Event::StreamEnded => {
                // Stream complete - send completion notification and respond
                cx.send_notification(ResponseCompleteNotification {
                    request_id: request_id.clone(),
                })?;
                request_cx.respond(ProvideResponseResponse {})?;
                return Ok(StreamEndReason::Complete);
            }
            Event::Cancelled => {
                // Cancelled - respond with error
                tracing::debug!(?request_id, "streaming cancelled");
                request_cx.respond_with_error(sacp::Error::new(
                    ERROR_CODE_CANCELLED,
                    "Request cancelled",
                ))?;
                return Ok(StreamEndReason::Cancelled);
            }
        }
    }

    Ok(StreamEndReason::Complete)
}

impl JrMessageHandler for LmBackendHandler {
    type Link = LmBackendToVsCode;

    fn describe_chain(&self) -> impl std::fmt::Debug {
        "LmBackendHandler"
    }

    async fn handle_message(
        &mut self,
        message: MessageCx,
        cx: JrConnectionCx<Self::Link>,
    ) -> Result<Handled<MessageCx>, sacp::Error> {
        tracing::trace!(?message, "handle_message");
        MatchMessage::new(message)
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

                // Get the request ID from the request context for notifications
                let request_id = request_cx.id().clone();

                // Find session with longest matching prefix
                let (session_idx, prefix_len) = self
                    .sessions
                    .iter()
                    .enumerate()
                    .filter_map(|(i, s)| s.prefix_match_len(&req.messages).map(|len| (i, len)))
                    .max_by_key(|(_, len)| *len)
                    .unwrap_or((usize::MAX, 0));

                // Get or create session
                let session_data = if session_idx < self.sessions.len() {
                    let session_data = &mut self.sessions[session_idx];
                    tracing::debug!(
                        session_id = %session_data.actor.session_id(),
                        prefix_len,
                        "continuing existing session"
                    );
                    session_data
                } else {
                    let actor = SessionActor::spawn(&cx, req.agent.clone())?;
                    self.sessions.push(SessionData {
                        actor,
                        state: SessionState::Idle,
                    });
                    self.sessions.last_mut().unwrap()
                };

                // Compute new messages (everything after the matched prefix)
                let new_messages = &req.messages[prefix_len..];
                tracing::debug!(
                    session_id = %session_data.actor.session_id(),
                    new_message_count = new_messages.len(),
                    "sending new messages to session"
                );

                session_data.receive_messages(&new_messages)
            })
            .await
            .if_notification(async |notification: CancelNotification| {
                tracing::debug!(?notification, "CancelNotification");

                // Find the session streaming this request
                if let Some(session_data) = self
                    .sessions
                    .iter_mut()
                    .find(|s| s.is_streaming_request(&notification.request_id))
                {
                    session_data.state.cancel();
                    tracing::debug!(
                        session_id = %session_data.actor.session_id(),
                        "cancelled streaming response"
                    );
                } else {
                    tracing::warn!(
                        request_id = ?notification.request_id,
                        "cancel notification for unknown request"
                    );
                }

                Ok(())
            })
            .await
            .otherwise(async |message| match message {
                MessageCx::Request(request, request_cx) => {
                    tracing::warn!("unknown request method: {}", request.method());
                    request_cx.respond_with_error(sacp::Error::method_not_found())
                }
                MessageCx::Notification(notif) => {
                    tracing::warn!("unexpected notification: {}", notif.method());
                    Ok(())
                }
            })
            .await?;

        Ok(Handled::Yes)
    }
}

// ============================================================================
// Component Implementation
// ============================================================================

/// The LM backend component that can be used with sacp's Component infrastructure.
pub struct LmBackend {
    handler: LmBackendHandler,
}

impl LmBackend {
    pub fn new() -> Self {
        Self {
            handler: LmBackendHandler::new(),
        }
    }
}

impl Default for LmBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl sacp::Component<LmBackendToVsCode> for LmBackend {
    async fn serve(
        self,
        client: impl sacp::Component<VsCodeToLmBackend>,
    ) -> Result<(), sacp::Error> {
        LmBackendToVsCode::builder()
            .with_handler(self.handler)
            .serve(client)
            .await
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

    LmBackend::new().serve(stdio).await?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[tokio::test]
    async fn test_provide_info() -> Result<(), sacp::Error> {
        VsCodeToLmBackend::builder()
            .connect_to(LmBackend::new())?
            .run_until(async |cx| {
                let response = cx
                    .send_request(ProvideInfoRequest { silent: false })
                    .block_task()
                    .await?;

                expect![[r#"
                    ProvideInfoResponse {
                        models: [
                            ModelInfo {
                                id: "symposium-eliza",
                                name: "Symposium (Eliza)",
                                family: "symposium",
                                version: "1.0.0",
                                max_input_tokens: 100000,
                                max_output_tokens: 100000,
                                capabilities: ModelCapabilities {
                                    tool_calling: true,
                                },
                            },
                        ],
                    }
                "#]]
                .assert_debug_eq(&response);

                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn test_provide_token_count() -> Result<(), sacp::Error> {
        VsCodeToLmBackend::builder()
            .connect_to(LmBackend::new())?
            .run_until(async |cx| {
                let response = cx
                    .send_request(ProvideTokenCountRequest {
                        model_id: "symposium-eliza".to_string(),
                        text: "Hello, world!".to_string(),
                    })
                    .block_task()
                    .await?;

                expect![[r#"
                    ProvideTokenCountResponse {
                        count: 3,
                    }
                "#]]
                .assert_debug_eq(&response);

                Ok(())
            })
            .await
    }

    // TODO: Add integration tests that spawn a real agent process
    // The chat_response and session_continuation tests have been removed
    // because they relied on the old in-process Eliza implementation.
    // With the new architecture, the session actor spawns an external
    // ACP agent process, which requires different test infrastructure.

    #[test]
    fn test_agent_definition_eliza_serialization() {
        use super::session_actor::AgentDefinition;

        let agent = AgentDefinition::Eliza {
            deterministic: true,
        };
        let json = serde_json::to_string_pretty(&agent).unwrap();
        println!("Eliza:\n{}", json);

        // Should serialize as {"eliza": {"deterministic": true}}
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("eliza").is_some());
        assert_eq!(parsed["eliza"]["deterministic"], true);
    }

    #[test]
    fn test_agent_definition_mcp_server_serialization() {
        use super::session_actor::AgentDefinition;
        use sacp::schema::{McpServer, McpServerStdio};

        let server = McpServer::Stdio(McpServerStdio::new("test", "echo"));
        let agent = AgentDefinition::McpServer(server);
        let json = serde_json::to_string_pretty(&agent).unwrap();
        println!("McpServer:\n{}", json);

        // Should serialize as {"mcp_server": {name, command, args, env}}
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("mcp_server").is_some());
        assert_eq!(parsed["mcp_server"]["name"], "test");
        assert_eq!(parsed["mcp_server"]["command"], "echo");
    }
}
