//! Session actor for VS Code Language Model Provider
//!
//! Each session actor manages a single conversation with an LLM backend (currently Eliza,
//! eventually an ACP agent). The actor pattern isolates session state and enables clean
//! cancellation via channel closure.

use elizacp::eliza::Eliza;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::{LmBackendToVsCode, Message, ResponsePart};

/// Message sent to the session actor
struct SessionMessage {
    /// New messages to process (not the full history, just what's new)
    new_messages: Vec<Message>,
    /// Channel for streaming response parts back
    reply_tx: mpsc::UnboundedSender<ResponsePart>,
}

/// Handle for communicating with a session actor.
///
/// This follows the Tokio actor pattern: the handle owns a sender channel and provides
/// methods for interacting with the actor. The actor itself runs in a spawned task.
pub struct SessionActor {
    tx: mpsc::UnboundedSender<SessionMessage>,
    /// Unique identifier for this session (for logging)
    session_id: Uuid,
    /// The message history this session has processed
    history: Vec<Message>,
}

impl SessionActor {
    /// Spawn a new session actor.
    ///
    /// Creates the actor's mailbox and spawns the run loop. Returns a handle
    /// for sending messages to the actor.
    ///
    /// If `deterministic` is true, uses deterministic Eliza responses (for testing).
    pub fn spawn(
        cx: &sacp::JrConnectionCx<LmBackendToVsCode>,
        deterministic: bool,
    ) -> Result<Self, sacp::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let session_id = Uuid::new_v4();
        let eliza = if deterministic {
            Eliza::new_deterministic()
        } else {
            Eliza::new()
        };
        tracing::info!(%session_id, "spawning new session actor");
        cx.spawn(Self::run(rx, eliza, session_id))?;
        Ok(Self {
            tx,
            session_id,
            history: Vec::new(),
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
    pub fn send_prompt(
        &mut self,
        new_messages: Vec<Message>,
    ) -> mpsc::UnboundedReceiver<ResponsePart> {
        let (reply_tx, reply_rx) = mpsc::unbounded_channel();

        // Update our history with what we're sending
        self.history.extend(new_messages.clone());

        // Send to the actor (ignore errors - actor may have died)
        let _ = self.tx.send(SessionMessage {
            new_messages,
            reply_tx,
        });

        reply_rx
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
        mut rx: mpsc::UnboundedReceiver<SessionMessage>,
        mut eliza: Eliza,
        session_id: Uuid,
    ) -> Result<(), sacp::Error> {
        tracing::debug!(%session_id, "session actor started");
        while let Some(msg) = rx.recv().await {
            let new_message_count = msg.new_messages.len();
            tracing::debug!(%session_id, new_message_count, "received new messages");

            // Process each new message
            for (i, message) in msg.new_messages.into_iter().enumerate() {
                tracing::trace!(%session_id, message_index = i, role = %message.role, "processing message");
                if message.role == "user" {
                    let user_text = message.text();
                    let response = eliza.respond(&user_text);
                    tracing::debug!(%session_id, %user_text, %response, "eliza response");

                    // Stream response in chunks
                    for chunk in response.chars().collect::<Vec<_>>().chunks(5) {
                        let text: String = chunk.iter().collect();
                        if msg
                            .reply_tx
                            .send(ResponsePart::Text { value: text })
                            .is_err()
                        {
                            // Channel closed = request was cancelled
                            tracing::debug!(%session_id, "reply channel closed, request cancelled");
                            break;
                        }
                    }
                }
            }
            tracing::debug!(%session_id, "finished processing request");
            // reply_tx drops here when msg goes out of scope, signaling completion
        }
        tracing::debug!(%session_id, "session actor shutting down");
        Ok(())
    }
}
