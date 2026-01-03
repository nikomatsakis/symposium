//! Session actor for VS Code Language Model Provider
//!
//! Each session actor manages a single conversation with an ACP agent. The actor pattern
//! isolates session state and enables clean cancellation via channel closure.

use elizacp::ElizaAgent;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::{mpsc, oneshot};
use futures::stream::Peekable;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt};
use futures_concurrency::future::FutureExt as _;
use sacp::schema::{
    ToolCall, ToolCallId, ToolCallLocation, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
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
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::pin::Pin;
use uuid::Uuid;

use crate::vscodelm::SYMPOSIUM_AGENT_ACTION;

use super::{ContentPart, LmBackendToVsCode, Message};

/// Defines which agent backend to use for a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// A request made of the model
#[derive(Debug)]
struct ModelRequest {
    /// New messages to process (not the full history, just what's new)
    new_messages: Vec<Message>,

    /// Channel for streaming response parts back.
    /// Drop to indicate that we are waiting for a new request before continuing.
    prompt_tx: mpsc::UnboundedSender<ContentPart>,

    /// Receiving `()` on this channel indicates cancellation.
    cancel_rx: oneshot::Receiver<()>,
}

/// Information about a pending permission request
struct PendingPermission {
    /// The tool call ID we emitted to VS Code
    tool_call_id: String,
    /// Channel to send the decision back to the agent loop
    decision_tx: oneshot::Sender<bool>,
}

/// State of the session from the handler's perspective
#[derive(Debug)]
pub enum ActorState {
    /// Ready for a new prompt
    Idle,
    /// Awaiting permission decision from VS Code
    AwaitingPermission {
        /// The tool call ID we're waiting for
        tool_call_id: String,
    },
}

/// Handle for communicating with a session actor.
///
/// This follows the Tokio actor pattern: the handle owns a sender channel and provides
/// methods for interacting with the actor. The actor itself runs in a spawned task.
pub struct SessionActor {
    tx: mpsc::UnboundedSender<ModelRequest>,
    /// Unique identifier for this session (for logging)
    session_id: Uuid,
    /// The message history this session has processed
    history: Vec<Message>,
    /// The agent definition (stored for future prefix matching)
    #[allow(dead_code)]
    agent_definition: AgentDefinition,
    /// Current state of the actor
    state: ActorState,
}

impl SessionActor {
    /// Spawn a new session actor.
    ///
    /// Creates the actor's mailbox and spawns the run loop. Returns a handle
    /// for sending messages to the actor.
    pub fn spawn(
        cx: &sacp::JrConnectionCx<LmBackendToVsCode>,
        agent_definition: AgentDefinition,
    ) -> Result<Self, sacp::Error> {
        let (tx, rx) = mpsc::unbounded();
        let session_id = Uuid::new_v4();
        tracing::info!(%session_id, ?agent_definition, "spawning new session actor");
        cx.spawn(Self::run(rx, agent_definition.clone(), session_id))?;
        Ok(Self {
            tx,
            session_id,
            history: Vec::new(),
            agent_definition,
            state: ActorState::Idle,
        })
    }

    /// Returns the session ID (for logging).
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Send new content to the actor, returns a receiver for streaming response.
    ///
    /// The caller should stream from the returned receiver until it closes,
    /// which signals that the actor has finished processing.
    ///
    /// To cancel the request, simply drop the receiver - the actor will see
    /// send failures and stop processing.
    pub fn send_prompt(&mut self, new_messages: Vec<Message>) -> Result<ActivePrompt, sacp::Error> {
        let (prompt_tx, prompt_rx) = mpsc::unbounded();
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Update our history with what we're sending
        self.history.extend(new_messages.clone());

        // Send to the actor (ignore errors - actor may have died)
        self.tx
            .unbounded_send(ModelRequest {
                new_messages,
                prompt_tx,
                cancel_rx,
            })
            .map_err(sacp::util::internal_error)?;

        Ok(ActivePrompt {
            cancel_tx,
            prompt_rx,
        })
    }

    /// Get the current state of the actor.
    pub fn state(&self) -> &ActorState {
        &self.state
    }

    /// Set the actor state to awaiting permission.
    pub fn set_awaiting_permission(&mut self, tool_call_id: String) {
        self.state = ActorState::AwaitingPermission { tool_call_id };
    }

    /// Check if incoming messages extend our history.
    ///
    /// Returns the number of matching prefix messages, or None if the incoming
    /// messages don't start with our history.
    pub fn prefix_match_len(&self, messages: &[Message]) -> Option<usize> {
        if messages.len() < self.history.len() {
            return None;
        }
        if self
            .history
            .iter()
            .zip(messages.iter())
            .all(|(a, b)| a == b)
        {
            Some(self.history.len())
        } else {
            None
        }
    }

    /// The actor's main run loop.
    async fn run(
        actor_rx: mpsc::UnboundedReceiver<ModelRequest>,
        agent_definition: AgentDefinition,
        session_id: Uuid,
    ) -> Result<(), sacp::Error> {
        tracing::debug!(%session_id, "session actor starting");

        match agent_definition {
            AgentDefinition::Eliza { deterministic } => {
                let agent = ElizaAgent::new(deterministic);
                Self::run_with_agent(actor_rx, agent, session_id).await
            }
            AgentDefinition::McpServer(config) => {
                let agent = AcpAgent::new(config);
                Self::run_with_agent(actor_rx, agent, session_id).await
            }
        }
    }

    /// Run the session with a specific agent component.
    async fn run_with_agent(
        actor_rx: mpsc::UnboundedReceiver<ModelRequest>,
        agent: impl Component<sacp::link::AgentToClient>,
        session_id: Uuid,
    ) -> Result<(), sacp::Error> {
        ClientToAgent::builder()
            .connect_to(agent)?
            .run_until(async |cx| {
                tracing::debug!(%session_id, "connected to agent, initializing");

                // Initialize the agent
                let _init_response = cx
                    .send_request(InitializeRequest::new(ProtocolVersion::LATEST))
                    .block_task()
                    .await?;

                tracing::debug!(%session_id, "agent initialized, creating session");

                Self::run_with_cx(actor_rx, cx, session_id).await
            })
            .await
    }

    async fn run_with_cx(
        actor_rx: mpsc::UnboundedReceiver<ModelRequest>,
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

        // Process messages from the handler
        let mut actor_rx = actor_rx.peekable();
        while let Some(mut request) = actor_rx.next().await {
            let new_message_count = request.new_messages.len();
            tracing::debug!(%session_id, new_message_count, "received new messages");

            // Build prompt from new messages
            // For now, just concatenate user messages
            let prompt_text: String = request
                .new_messages
                .iter()
                .filter(|m| m.role == "user")
                .map(|m| m.text())
                .collect::<Vec<_>>()
                .join("\n");

            if prompt_text.is_empty() {
                tracing::debug!(%session_id, "no user messages, skipping");
                continue;
            }

            tracing::debug!(%session_id, %prompt_text, "sending prompt to agent");
            session.send_prompt(&prompt_text)?;

            // Read updates from the prompt.
            let canceled = loop {
                // Wait for either an update to the session
                // or the cancellation message.
                let cancel_rx = &mut request.cancel_rx;
                let update = session
                    .read_update()
                    .map_ok(Some)
                    .race(cancel_rx.map(|_| Ok(None)))
                    .await?;

                let Some(update) = update else {
                    // Cancelled.
                    break true;
                };

                match update {
                    sacp::SessionMessage::SessionMessage(message) => {
                        match Self::process_session_message(
                            request,
                            message,
                            &mut actor_rx,
                            session_id,
                        )
                        .await?
                        {
                            Some(r) => request = r,
                            None => break true,
                        }
                    }
                    sacp::SessionMessage::StopReason(stop_reason) => {
                        tracing::debug!(
                            %session_id,
                            ?stop_reason,
                            "agent turn complete"
                        );
                        break false;
                    }
                    other => {
                        tracing::trace!(
                            %session_id,
                            ?other,
                            "ignoring session message"
                        );
                    }
                }
            };

            // FIXME: if the agent has sent a request permission here, we might do something a bit odd.
            // We should really read out the updates that we received before sending out the cancelation.
            // There is also an inherent race condition in the protocol, let's talk that out with
            // other folks -- if the client MUST respond with "canceled" to a request permission request,
            // but it may cancel BEFORE receiving the request permission request, what is it supposed to do?

            // If we got a cancelation, then inform the ACP agent the current prompt is canceled.
            // We'll then loop around and await another message.
            if canceled {
                cx.send_notification(sacp::schema::CancelNotification::new(
                    session.session_id().clone(),
                ))?;
            }
        }

        tracing::debug!(%session_id, "session actor shutting down");
        Ok(())
    }

    /// Process a single session message from the ACP agent.
    ///
    /// Returns `Some(result)` if we should exit the update loop, `None` to continue.
    async fn process_session_message(
        request: ModelRequest,
        message: MessageCx,
        actor_rx: Pin<&mut Peekable<mpsc::UnboundedReceiver<ModelRequest>>>,
        session_id: Uuid,
    ) -> Result<Option<ModelRequest>, sacp::Error> {
        use sacp::util::MatchMessage;

        let mut cancel = false;

        macro_rules! control_flow_break {
            () => {{
                cancel = true;
                Ok(())
            }};
        }

        MatchMessage::new(message)
            .if_notification(async |notif: SessionNotification| {
                // Session-updates: send them back through `reply_tx` so they
                // can be posted to vscode.
                if let SessionUpdate::AgentMessageChunk(chunk) = notif.update {
                    let text = content_block_to_string(&chunk.content);
                    if !text.is_empty() {
                        if let Err(_) = request
                            .prompt_tx
                            .unbounded_send(ContentPart::Text { value: text })
                        {
                            tracing::debug!(
                                %session_id,
                                "reply channel closed, request cancelled"
                            );
                            return control_flow_break!();
                        }
                    }
                }
                Ok(())
            })
            .await
            .if_request(async |perm_request: RequestPermissionRequest, request_cx| {
                // Permission requests: these fall into two cases.
                //
                // 1. Requests to use a tool provided by VSCode (not yet handled).
                // 2. Requests to use a tool internal to the agent (or one of the proxies).
                //
                // The challenge is that, the VSCode language model interface that we
                // are implementing expects the model to simply *use* tools, not to *ask permission*.
                // VSCode expects to handle permission itself. We bridge the gap in two different
                // ways, depending on which case it is.
                //
                // For case 1, we always approve. This should result in the agent actually
                // performing the tool call. At that point, we'll ferry the call to VSCode.
                // VSCode will manage the approval and so forth.
                //
                // For case 2, we send the approval request back to the main vscode actor.
                // It will translate the "approval request" into a *use* of a special tool
                // that is a no-op and just exists to ask permission. If the user approves,
                // the tool will execute and produce a `()` result, which will get sent back
                // to this actor as "approval". Otherwise, the session will be canceled.

                tracing::debug!(
                    %session_id,
                    ?perm_request,
                    "received permission request from agent"
                );

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
                                    locations,
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

                let tool_call = ContentPart::ToolCall {
                    tool_call_id: tool_call_id.to_string(),
                    tool_name: SYMPOSIUM_AGENT_ACTION.to_string(),
                    parameters: serde_json::json!({
                        "kind": kind,
                        "title": title,
                        "raw_input": raw_input,
                    }),
                };
                tracing::info!(?tool_call, "requesting tool permission");

                // Emit tool call to VS Code
                if let Err(_) = request.prompt_tx.unbounded_send(tool_call) {
                    // Session was canceled.
                    tracing::debug!(
                        %session_id,
                        "reply channel closed, request cancelled"
                    );
                    return control_flow_break!();
                }

                // Drop the request on our side to signal to VSCode that it should
                // resume.
                drop(request);

                // Wait for VSCode to respond. If the stream ends,
                // just cancel.
                let Some(peek_request) = actor_rx.peek().await else {
                    return control_flow_break!();
                };
                tracing::debug!(?peek_request, "next request received");

                // Check if this request now includes a "tool call".
                // We expect two new messages:
                //
                // 1. An Assistant message** containing:
                // - Any text we streamed before the tool call
                // - The `LanguageModelToolCallPart` we emitted
                // 2. **A User message** containing:
                // - `LanguageModelToolResultPart` with the matching `callId` and result content
                //
                // If the request does NOT include a tool call, then we return a break.
                // This will cause the outer loop to execute around and "start over", essentially,
                // treating these new messages as just new messages.
                if !peek_request.ends_in_tool_call_response(tool_call_id) {
                    return control_flow_break!();
                }

                // Wait to hear about the decision.
                //
                // If the request is denied, send back cancellation -- that's the only thing that
                // language models in vscode can do.
                match decision_rx.await {
                    Ok(()) => (),
                    Err(_) => {
                        // Session was canceled.
                        tracing::debug!(
                            %session_id,
                            "permission denied, request cancelled"
                        );
                        request_cx.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Cancelled,
                        ))?;
                        return control_flow_break!();
                    }
                }

                // Requested approved! Look for a "approve once" option.
                let approve_once_outcome = options
                    .into_iter()
                    .filter(|option| match option.kind {
                        sacp::schema::PermissionOptionKind::AllowOnce => true,
                        _ => false,
                    })
                    .map(|option| {
                        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                            option.option_id,
                        ))
                    })
                    .next();

                match approve_once_outcome {
                    Some(o) => request_cx.respond(RequestPermissionResponse::new(o))?,
                    None => {
                        // If the tool didn't give us the option to approve once, wtf.
                        // Just cancel the tool request I guess.
                        request_cx.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Cancelled,
                        ))?;
                        return control_flow_break!();
                    }
                }

                Ok(())
            })
            .await
            .otherwise(async |message| {
                match message {
                    MessageCx::Request(request, request_cx) => {
                        tracing::warn!(
                            %session_id,
                            method = request.method(),
                            "unknown request from agent"
                        );
                        request_cx
                            .respond_with_error(sacp::util::internal_error("unknown request"))?;
                    }
                    MessageCx::Notification(notif) => {
                        tracing::trace!(
                            %session_id,
                            method = notif.method(),
                            "ignoring unhandled notification"
                        );
                    }
                }
                Ok(())
            })
            .await?;

        if cancel {
            Ok(None)
        } else {
            Ok(Some(request))
        }
    }
}

/// Result of processing agent updates
enum UpdateLoopResult {
    /// Turn completed normally
    TurnComplete,
    /// Awaiting permission decision from VS Code
    AwaitingPermission { tool_call_id: String },
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

/// Struct which, when dropped, will signal the session actor
/// to stop working on the prompt.
pub struct ActivePrompt {
    cancel_tx: Option<oneshot::Sender<()>>,
    prompt_rx: mpsc::UnboundedReceiver<ContentPart>,
}

impl ActivePrompt {
    /// Receiver for [`SessionToCodeMessage`] that being sent in response to the prompt.
    pub fn prompt_rx(&mut self) -> &mut mpsc::UnboundedReceiver<ContentPart> {
        &mut self.prompt_rx
    }
}

impl Drop for ActivePrompt {
    fn drop(&mut self) {
        let cancel_tx = self.cancel_tx.take().expect("not yet dropped");
        let _ = cancel_tx.send(()); // ignore errors in response
    }
}

mod request_response;
