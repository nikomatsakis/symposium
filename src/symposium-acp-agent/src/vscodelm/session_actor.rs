//! Session actor for VS Code Language Model Provider
//!
//! Each session actor manages a single conversation with an ACP agent. It receives
//! messages from the HistoryActor and sends response parts back to it.

use elizacp::ElizaAgent;
use futures::channel::{mpsc, oneshot};
use futures::stream::Peekable;
use futures::{FutureExt, StreamExt, TryFutureExt};
use futures_concurrency::future::FutureExt as _;
use sacp::schema::{
    ToolCall, ToolCallContent, ToolCallId, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields,
};
use sacp::JrConnectionCx;
use sacp::{
    schema::{
        InitializeRequest, ProtocolVersion, RequestPermissionOutcome, RequestPermissionRequest,
        RequestPermissionResponse, SelectedPermissionOutcome, SessionNotification, SessionUpdate,
    },
    ClientToAgent, Component, MessageCx,
};
use sacp_tokio::AcpAgent;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use uuid::Uuid;

use super::history_actor::{HistoryActorHandle, SessionToHistoryMessage};
use super::{ContentPart, Message, ROLE_USER, SYMPOSIUM_AGENT_ACTION};

/// Tracks the state of tool calls and renders them to markdown.
///
/// Tool calls arrive as an initial `ToolCall` followed by `ToolCallUpdate` messages.
/// We accumulate the state and re-render the markdown on each update, streaming
/// the result to VS Code as text parts.
#[derive(Debug, Default)]
struct ToolCallTracker {
    /// Current state of each tool call, keyed by tool_call_id
    tool_calls: HashMap<ToolCallId, ToolCallState>,
}

/// Accumulated state for a single tool call
#[derive(Debug, Clone)]
struct ToolCallState {
    title: String,
    status: ToolCallStatus,
    content: Vec<ToolCallContent>,
}

impl ToolCallTracker {
    fn new() -> Self {
        Self::default()
    }

    /// Process an initial tool call notification
    fn handle_tool_call(&mut self, tool_call: ToolCall) -> String {
        let state = ToolCallState {
            title: tool_call.title,
            status: tool_call.status,
            content: tool_call.content,
        };
        self.tool_calls
            .insert(tool_call.tool_call_id.clone(), state.clone());
        self.render_tool_call(&state)
    }

    /// Process a tool call update notification
    fn handle_tool_call_update(&mut self, update: ToolCallUpdate) -> Option<String> {
        let state = self.tool_calls.get_mut(&update.tool_call_id)?;

        // Apply updates
        if let Some(title) = update.fields.title {
            state.title = title;
        }
        if let Some(status) = update.fields.status {
            state.status = status;
        }
        if let Some(content) = update.fields.content {
            state.content = content;
        }

        // Clone to avoid borrow conflict
        let state = state.clone();
        Some(self.render_tool_call(&state))
    }

    /// Render a tool call state to markdown
    fn render_tool_call(&self, state: &ToolCallState) -> String {
        let mut output = String::new();

        // Status indicator
        let status_icon = match state.status {
            ToolCallStatus::Pending => "⏳",
            ToolCallStatus::InProgress => "⚙️",
            ToolCallStatus::Completed => "✅",
            ToolCallStatus::Failed => "❌",
            _ => "•",
        };

        // Header with title
        output.push_str(&format!("{} **{}**\n", status_icon, state.title));

        // Content - render in a long code fence to allow nested fences
        if !state.content.is_empty() {
            output.push_str("``````````\n");
            for content in &state.content {
                output.push_str(&tool_call_content_to_string(content));
            }
            // Ensure content ends with newline before closing fence
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("``````````\n");
        }

        output
    }

    /// Clear all tracked tool calls (call at end of turn)
    fn clear(&mut self) {
        self.tool_calls.clear();
    }
}

/// Convert tool call content to a string representation
fn tool_call_content_to_string(content: &ToolCallContent) -> String {
    match content {
        ToolCallContent::Content(c) => {
            // Content contains a ContentBlock
            content_block_to_string(&c.content)
        }
        ToolCallContent::Diff(diff) => {
            format!(
                "--- {}\n+++ {}\n{}",
                diff.path.display(),
                diff.path.display(),
                diff.new_text
            )
        }
        ToolCallContent::Terminal(terminal) => {
            format!("[Terminal: {}]", terminal.terminal_id)
        }
        _ => "[Unknown content]".to_string(),
    }
}

/// Defines which agent backend to use for a session.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDefinition {
    /// Use the in-process Eliza chatbot (for testing)
    Eliza {
        #[serde(default)]
        deterministic: bool,
    },
    /// Spawn an external ACP agent process
    McpServer(sacp::schema::McpServer),
}

/// Messages sent to SessionActor from HistoryActor.
#[derive(Debug)]
pub struct SessionRequest {
    /// New messages to process
    pub messages: Vec<Message>,
    /// Whether this request represents a cancellation of previous work
    pub canceled: bool,
    /// Per-request state that travels with the request
    pub state: RequestState,
}

/// Per-request state that needs to be passed through message processing.
/// This is bundled together because both values can change between requests.
#[derive(Debug)]
pub struct RequestState {
    /// Cancelation channel for this request
    pub cancel_rx: oneshot::Receiver<()>,
    /// Whether the internal tool (symposium-agent-action) is available.
    /// If false, all permission requests should be auto-denied.
    pub has_internal_tool: bool,
}

impl RequestState {
    /// Returns a future that completes when cancellation is requested.
    /// The future resolves to `Ok(None)` to signal cancellation in select/race patterns.
    pub fn cancellation(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<sacp::SessionMessage>, sacp::Error>> + '_
    {
        (&mut self.cancel_rx).map(|_| Ok(None))
    }
}

/// Handle for communicating with a session actor.
pub struct SessionActor {
    /// Channel to send requests to the actor
    tx: mpsc::UnboundedSender<SessionRequest>,
    /// Unique identifier for this session
    session_id: Uuid,
}

impl SessionActor {
    /// Spawn a new session actor.
    pub fn spawn(
        history_handle: HistoryActorHandle,
        agent_definition: AgentDefinition,
    ) -> Result<Self, sacp::Error> {
        let (tx, rx) = mpsc::unbounded();
        let session_id = Uuid::new_v4();

        tracing::info!(%session_id, ?agent_definition, "spawning new session actor");

        // Spawn the actor task
        tokio::spawn(Self::run(rx, history_handle, agent_definition, session_id));

        Ok(Self { tx, session_id })
    }

    /// Returns the session ID.
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Send messages to the session actor.
    pub fn send_messages(
        &self,
        messages: Vec<Message>,
        canceled: bool,
        cancel_rx: oneshot::Receiver<()>,
        has_internal_tool: bool,
    ) {
        let _ = self.tx.unbounded_send(SessionRequest {
            messages,
            canceled,
            state: RequestState {
                cancel_rx,
                has_internal_tool,
            },
        });
    }

    /// The actor's main run loop.
    async fn run(
        request_rx: mpsc::UnboundedReceiver<SessionRequest>,
        history_handle: HistoryActorHandle,
        agent_definition: AgentDefinition,
        session_id: Uuid,
    ) -> Result<(), sacp::Error> {
        tracing::debug!(%session_id, "session actor starting");

        let result = match agent_definition {
            AgentDefinition::Eliza { deterministic } => {
                let agent = ElizaAgent::new(deterministic);
                Self::run_with_agent(request_rx, history_handle.clone(), agent, session_id).await
            }
            AgentDefinition::McpServer(config) => {
                let agent = AcpAgent::new(config);
                Self::run_with_agent(request_rx, history_handle.clone(), agent, session_id).await
            }
        };

        if let Err(ref e) = result {
            history_handle
                .send_from_session(session_id, SessionToHistoryMessage::Error(e.to_string()))?;
        }

        result
    }

    /// Run the session with a specific agent component.
    async fn run_with_agent(
        request_rx: mpsc::UnboundedReceiver<SessionRequest>,
        history_handle: HistoryActorHandle,
        agent: impl Component<sacp::link::AgentToClient>,
        session_id: Uuid,
    ) -> Result<(), sacp::Error> {
        ClientToAgent::builder()
            .connect_to(agent)?
            .run_until(async |cx| {
                tracing::debug!(%session_id, "connected to agent, initializing");

                let _init_response = cx
                    .send_request(InitializeRequest::new(ProtocolVersion::LATEST))
                    .block_task()
                    .await?;

                tracing::debug!(%session_id, "agent initialized, creating session");

                Self::run_with_cx(request_rx, history_handle, cx, session_id).await
            })
            .await
    }

    async fn run_with_cx(
        request_rx: mpsc::UnboundedReceiver<SessionRequest>,
        history_handle: HistoryActorHandle,
        cx: JrConnectionCx<ClientToAgent>,
        session_id: Uuid,
    ) -> Result<(), sacp::Error> {
        // Create a session
        let mut session = cx
            .build_session(PathBuf::from("."))
            .block_task()
            .start_session()
            .await?;

        tracing::debug!(%session_id, "session created, waiting for messages");

        let mut request_rx = request_rx.peekable();
        let mut tool_call_tracker = ToolCallTracker::new();

        while let Some(request) = request_rx.next().await {
            let new_message_count = request.messages.len();
            tracing::debug!(%session_id, new_message_count, canceled = request.canceled, "received request");

            let SessionRequest {
                messages,
                canceled: _,
                state: mut request_state,
            } = request;

            // Build prompt from messages
            let prompt_text: String = messages
                .iter()
                .filter(|m| m.role == ROLE_USER)
                .map(|m| m.text())
                .collect::<Vec<_>>()
                .join("\n");

            if prompt_text.is_empty() {
                tracing::debug!(%session_id, "no user messages, skipping");
                history_handle.send_from_session(session_id, SessionToHistoryMessage::Complete)?;
                continue;
            }

            tracing::debug!(%session_id, %prompt_text, "sending prompt to agent");
            session.send_prompt(&prompt_text)?;

            // Read updates from the agent
            let canceled = loop {
                // Wait for either an update or a cancellation
                let update = session
                    .read_update()
                    .map_ok(Some)
                    .race(request_state.cancellation())
                    .await?;

                let Some(update) = update else {
                    // Canceled
                    break true;
                };

                match update {
                    sacp::SessionMessage::SessionMessage(message) => {
                        let new_state = Self::process_session_message(
                            message,
                            &history_handle,
                            &mut request_rx,
                            request_state,
                            &mut tool_call_tracker,
                            session_id,
                        )
                        .await?;

                        match new_state {
                            Some(s) => request_state = s,
                            None => break true,
                        }
                    }
                    sacp::SessionMessage::StopReason(stop_reason) => {
                        tracing::debug!(%session_id, ?stop_reason, "agent turn complete");
                        break false;
                    }
                    other => {
                        tracing::trace!(%session_id, ?other, "ignoring session message");
                    }
                }
            };

            if canceled {
                cx.send_notification(sacp::schema::CancelNotification::new(
                    session.session_id().clone(),
                ))?;
            } else {
                // Turn completed normally
                history_handle.send_from_session(session_id, SessionToHistoryMessage::Complete)?;
            }

            // Clear tool call state for next turn
            tool_call_tracker.clear();
        }

        tracing::debug!(%session_id, "session actor shutting down");
        Ok(())
    }

    /// Process a single session message from the ACP agent.
    /// This will end the turn on the vscode side, so we consume the `request_state`.
    /// Returns `Some` with a new `RequestState` if tool use was approved (and sends that response to the agent).
    /// Returns `None` if tool use was declined; the outer loop should await a new prompt.
    async fn process_session_message(
        message: MessageCx,
        history_handle: &HistoryActorHandle,
        request_rx: &mut Peekable<mpsc::UnboundedReceiver<SessionRequest>>,
        request_state: RequestState,
        tool_call_tracker: &mut ToolCallTracker,
        session_id: Uuid,
    ) -> Result<Option<RequestState>, sacp::Error> {
        use sacp::util::MatchMessage;

        let has_internal_tool = request_state.has_internal_tool;
        let mut return_value = Some(request_state);

        MatchMessage::new(message)
            .if_notification(async |notif: SessionNotification| {
                match notif.update {
                    SessionUpdate::AgentMessageChunk(chunk) => {
                        let text = content_block_to_string(&chunk.content);
                        if !text.is_empty() {
                            history_handle.send_from_session(
                                session_id,
                                SessionToHistoryMessage::Part(ContentPart::Text { value: text }),
                            )?;
                        }
                    }
                    SessionUpdate::ToolCall(tool_call) => {
                        let markdown = tool_call_tracker.handle_tool_call(tool_call);
                        history_handle.send_from_session(
                            session_id,
                            SessionToHistoryMessage::Part(ContentPart::Text { value: markdown }),
                        )?;
                    }
                    SessionUpdate::ToolCallUpdate(update) => {
                        if let Some(markdown) = tool_call_tracker.handle_tool_call_update(update) {
                            history_handle.send_from_session(
                                session_id,
                                SessionToHistoryMessage::Part(ContentPart::Text { value: markdown }),
                            )?;
                        }
                    }
                    _ => {
                        // Ignore other update types
                    }
                }
                Ok(())
            })
            .await
            .if_request(async |perm_request: RequestPermissionRequest, request_cx| {
                tracing::debug!(%session_id, has_internal_tool, ?perm_request, "received permission request");

                // If the internal tool is not available, auto-deny all permission requests
                if !has_internal_tool {
                    tracing::info!(%session_id, "auto-denying permission request: internal tool not available");
                    request_cx.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ))?;
                    return Ok(());
                }

                let RequestPermissionRequest {
                    session_id: _,
                    tool_call:
                        ToolCallUpdate {
                            tool_call_id,
                            fields:
                                ToolCallUpdateFields {
                                    kind,
                                    status: _,
                                    title,
                                    content: _,
                                    locations: _,
                                    raw_input,
                                    raw_output: _,
                                    ..
                                },
                            meta: _,
                            ..
                        },
                    options,
                    meta: _,
                    ..
                } = perm_request;

                let tool_call_id_str = tool_call_id.to_string();

                let tool_call = ContentPart::ToolCall {
                    tool_call_id: tool_call_id_str.clone(),
                    tool_name: SYMPOSIUM_AGENT_ACTION.to_string(),
                    parameters: serde_json::json!({
                        "kind": kind,
                        "title": title,
                        "raw_input": raw_input,
                    }),
                };

                // Send tool call to history actor (which forwards to VS Code)
                history_handle.send_from_session(
                    session_id,
                    SessionToHistoryMessage::Part(tool_call),
                )?;

                // Signal completion so VS Code shows the confirmation UI
                history_handle.send_from_session(session_id, SessionToHistoryMessage::Complete)?;

                // Drop the cancel_rx because we just signaled completion.
                return_value = None;

                // Wait for the next request (which will have the tool result if approved)
                let Some(next_request) = Pin::new(&mut *request_rx).peek().await else {
                    request_cx.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ))?;
                    return Ok(());
                };

                // Check if canceled (history mismatch = rejection) or does not contain expected tool-use result
                if next_request.canceled || !next_request.messages[0].has_just_tool_result(&tool_call_id_str) {
                    tracing::debug!(%session_id, ?next_request, "permission denied, did not receive approval");
                    request_cx.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ))?;
                    return Ok(());
                }

                // Permission approved - find allow-once option and send.
                // If there is no such option, just cancel.
                let approve_once_outcome = options
                    .into_iter()
                    .find(|option| {
                        matches!(option.kind, sacp::schema::PermissionOptionKind::AllowOnce)
                    })
                    .map(|option| {
                        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                            option.option_id,
                        ))
                    });

                match approve_once_outcome {
                    Some(o) => request_cx.respond(RequestPermissionResponse::new(o))?,
                    None => {
                        request_cx.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Cancelled,
                        ))?;
                        return Ok(());
                    }
                }

                // Consume the request and use its state for the next iteration
                let SessionRequest { messages, canceled, state } = request_rx.next().await.expect("message is waiting");
                assert_eq!(canceled, false);
                assert_eq!(messages.len(), 1);
                return_value = Some(state);

                Ok(())
            })
            .await
            .otherwise(async |message| {
                match message {
                    MessageCx::Request(req, request_cx) => {
                        tracing::warn!(%session_id, method = req.method(), "unknown request");
                        request_cx.respond_with_error(sacp::util::internal_error("unknown request"))?;
                    }
                    MessageCx::Notification(notif) => {
                        tracing::trace!(%session_id, method = notif.method(), "ignoring notification");
                    }
                }
                Ok(())
            })
            .await?;

        Ok(return_value)
    }
}

/// Convert a content block to a string representation
fn content_block_to_string(block: &sacp::schema::ContentBlock) -> String {
    use sacp::schema::{ContentBlock, EmbeddedResourceResource};
    match block {
        ContentBlock::Text(text) => text.text.clone(),
        ContentBlock::Image(img) => format!("[Image: {}]", img.mime_type),
        ContentBlock::Audio(audio) => format!("[Audio: {}]", audio.mime_type),
        ContentBlock::ResourceLink(link) => link.uri.clone(),
        ContentBlock::Resource(resource) => match &resource.resource {
            EmbeddedResourceResource::TextResourceContents(text) => text.uri.clone(),
            EmbeddedResourceResource::BlobResourceContents(blob) => blob.uri.clone(),
            _ => "[Unknown resource type]".to_string(),
        },
        _ => "[Unknown content type]".to_string(),
    }
}

// TODO: request_response module is currently unused after refactoring to HistoryActor pattern.
// It may be useful later for a cleaner tool-call API, but needs to be updated for the new architecture.
// mod request_response;

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use sacp::schema::{ContentBlock, TextContent, ToolKind};

    #[test]
    fn test_tool_call_tracker_initial_call() {
        let mut tracker = ToolCallTracker::new();

        let tool_call = ToolCall::new("test-123", "Read src/main.rs")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::InProgress);

        let markdown = tracker.handle_tool_call(tool_call);

        expect![[r#"
            ⚙️ **Read src/main.rs**
        "#]]
        .assert_eq(&markdown);
    }

    #[test]
    fn test_tool_call_tracker_with_content() {
        let mut tracker = ToolCallTracker::new();

        let tool_call = ToolCall::new("test-456", "grep -n pattern file.rs")
            .kind(ToolKind::Search)
            .status(ToolCallStatus::Completed)
            .content(vec![ContentBlock::Text(TextContent::new(
                "10: let pattern = \"hello\";\n20: println!(\"{}\", pattern);",
            ))
            .into()]);

        let markdown = tracker.handle_tool_call(tool_call);

        expect![[r#"
            ✅ **grep -n pattern file.rs**
            ``````````
            10: let pattern = "hello";
            20: println!("{}", pattern);
            ``````````
        "#]]
        .assert_eq(&markdown);
    }

    #[test]
    fn test_tool_call_tracker_update() {
        let mut tracker = ToolCallTracker::new();

        // Initial call
        let tool_call = ToolCall::new("test-789", "Running cargo build")
            .kind(ToolKind::Execute)
            .status(ToolCallStatus::InProgress);
        tracker.handle_tool_call(tool_call);

        // Update with completion and content
        let update = ToolCallUpdate::new(
            "test-789",
            ToolCallUpdateFields::new()
                .status(ToolCallStatus::Completed)
                .content(vec![ContentBlock::Text(TextContent::new(
                    "Build succeeded!",
                ))
                .into()]),
        );

        let markdown = tracker.handle_tool_call_update(update).unwrap();

        expect![[r#"
            ✅ **Running cargo build**
            ``````````
            Build succeeded!
            ``````````
        "#]]
        .assert_eq(&markdown);
    }

    #[test]
    fn test_tool_call_tracker_unknown_id_returns_none() {
        let mut tracker = ToolCallTracker::new();

        let update = ToolCallUpdate::new(
            "unknown-id",
            ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
        );

        assert!(tracker.handle_tool_call_update(update).is_none());
    }
}
