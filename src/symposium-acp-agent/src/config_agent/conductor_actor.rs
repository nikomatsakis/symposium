//! Actor that owns and manages a conductor connection.
//!
//! This actor:
//! - Spawns and initializes the conductor
//! - Receives session messages from ConfigAgent via a channel
//! - Forwards messages to the conductor
//! - Forwards notifications from the conductor back to the client

use super::ConfigAgentMessage;
use crate::registry::ComponentSourceExt;
use crate::user_config::ModConfig;
use futures::channel::mpsc::UnboundedSender;
use sacp::schema::{
    InitializeRequest, McpServer, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse,
};
use sacp::{Agent, Client, Conductor, ConnectionTo, Dispatch, DynConnectTo, Responder};
use sacp_conductor::{ConductorImpl, McpBridgeMode};
use sacp_tokio::AcpAgent;
use std::path::PathBuf;
use symposium_recommendations::{ComponentSource, ModKind};
use tokio::sync::{mpsc, oneshot};

/// Messages that can be sent to the ConductorActor.
pub enum ConductorMessage {
    /// A new session request. The conductor will send NewSessionCreated to ConfigAgent.
    NewSession {
        request: NewSessionRequest,
        responder: Responder<NewSessionResponse>,
    },

    /// A prompt request for a session.
    Prompt {
        request: PromptRequest,
        responder: Responder<PromptResponse>,
    },

    /// Forward an arbitrary message to the conductor.
    ForwardMessage { message: Dispatch },

    /// Pause the conductor. It will stop processing messages until the returned
    /// oneshot is dropped or receives a value.
    ///
    /// The sender provides a channel to receive the resume signal sender.
    Pause {
        /// Channel to send the resume signal sender back to the caller.
        resume_tx_sender: oneshot::Sender<oneshot::Sender<()>>,
    },
}

/// Handle for communicating with a ConductorActor.
#[derive(Clone, Debug)]
pub struct ConductorHandle {
    tx: mpsc::Sender<ConductorMessage>,
}

impl ConductorHandle {
    /// Spawn a new conductor actor for the given agent and mods.
    ///
    /// Returns a handle for sending messages to the actor.
    pub async fn spawn(
        workspace_path: PathBuf,
        agent: ComponentSource,
        mods: Vec<ModConfig>,
        trace_dir: Option<&PathBuf>,
        config_agent_tx: UnboundedSender<ConfigAgentMessage>,
        connection: &ConnectionTo<Client>,
    ) -> Result<Self, sacp::Error> {
        tracing::debug!(?workspace_path, ?agent, ?mods, "ConductorHandle::spawn");

        // Create the channel for receiving messages
        let (tx, rx) = mpsc::channel(32);

        let handle = Self { tx: tx.clone() };

        connection.spawn(run_actor(
            workspace_path,
            agent,
            mods,
            trace_dir.cloned(),
            config_agent_tx,
            handle.clone(),
            rx,
        ))?;

        Ok(handle)
    }

    /// Send a new session request to the conductor.
    /// The conductor will send NewSessionCreated to ConfigAgent when done.
    pub async fn send_new_session(
        &self,
        request: NewSessionRequest,
        responder: Responder<NewSessionResponse>,
    ) -> Result<(), sacp::Error> {
        tracing::debug!(?request, "ConductorHandle::send_new_session");

        self.tx
            .send(ConductorMessage::NewSession { request, responder })
            .await
            .map_err(|_| sacp::util::internal_error("Conductor actor closed"))
    }

    /// Send a prompt request to the conductor.
    pub async fn send_prompt(
        &self,
        request: PromptRequest,
        responder: Responder<PromptResponse>,
    ) -> Result<(), sacp::Error> {
        self.tx
            .send(ConductorMessage::Prompt { request, responder })
            .await
            .map_err(|_| sacp::util::internal_error("Conductor actor closed"))
    }

    /// Forward an arbitrary message to the conductor.
    pub async fn forward_message(&self, message: Dispatch) -> Result<(), sacp::Error> {
        tracing::debug!(?message, "ConductorHandle::forward_message");

        self.tx
            .send(ConductorMessage::ForwardMessage { message })
            .await
            .map_err(|_| sacp::util::internal_error("Conductor actor closed"))
    }

    /// Pause the conductor. Returns a sender that, when dropped or sent to,
    /// will resume the conductor.
    ///
    /// While paused, the conductor will not process any messages from the
    /// downstream agent or accept new requests.
    pub async fn pause(&self) -> Result<oneshot::Sender<()>, sacp::Error> {
        tracing::debug!("ConductorHandle::pause");

        let (resume_tx_sender, resume_tx_receiver) = oneshot::channel();

        self.tx
            .send(ConductorMessage::Pause { resume_tx_sender })
            .await
            .map_err(|_| sacp::util::internal_error("Conductor actor closed"))?;

        resume_tx_receiver
            .await
            .map_err(|_| sacp::util::internal_error("Conductor actor closed"))
    }
}

/// Build proxy components from ComponentSources.
async fn build_proxies(
    mod_sources: Vec<ComponentSource>,
) -> Result<Vec<DynConnectTo<Conductor>>, sacp::Error> {
    let mut proxies = vec![];
    for source in &mod_sources {
        tracing::debug!(mod_name = %source.display_name(), "Resolving mod");
        let server = source.resolve().await.map_err(|e| {
            tracing::error!(
                mod_name = %source.display_name(),
                error = %e,
                "Failed to resolve mod"
            );
            sacp::util::internal_error(format!(
                "Failed to resolve {}: {}",
                source.display_name(),
                e
            ))
        })?;
        proxies.push(DynConnectTo::new(AcpAgent::new(server)));
    }

    Ok(proxies)
}

/// Build proxy components from ComponentSources.
async fn build_mcp_servers(
    mod_sources: Vec<ComponentSource>,
) -> Result<Vec<McpServer>, sacp::Error> {
    let mut servers = vec![];
    for source in &mod_sources {
        tracing::debug!(mod_name = %source.display_name(), "Resolving mod");
        let server = source.resolve().await.map_err(|e| {
            tracing::error!(
                mod_name = %source.display_name(),
                error = %e,
                "Failed to resolve mod"
            );
            sacp::util::internal_error(format!(
                "Failed to resolve {}: {}",
                source.display_name(),
                e
            ))
        })?;
        servers.push(server);
    }

    Ok(servers)
}

/// Get enabled proxies from the list
fn enabled_proxies(mods: &[ModConfig]) -> Vec<ComponentSource> {
    mods.iter()
        .filter(|m| m.enabled)
        .filter(|m| matches!(m.kind, ModKind::Proxy))
        .map(|m| m.source.clone())
        .collect()
}

/// Get enabled mcp servers from the list
fn enabled_mcp_servers(mods: &[ModConfig]) -> Vec<ComponentSource> {
    mods.iter()
        .filter(|m| m.enabled)
        .filter(|m| matches!(m.kind, ModKind::MCP))
        .map(|m| m.source.clone())
        .collect()
}

/// The main actor loop.
async fn run_actor(
    workspace_path: PathBuf,
    agent: ComponentSource,
    mods: Vec<ModConfig>,
    _trace_dir: Option<PathBuf>,
    config_agent_tx: UnboundedSender<ConfigAgentMessage>,
    self_handle: ConductorHandle,
    mut rx: mpsc::Receiver<ConductorMessage>,
) -> Result<(), sacp::Error> {
    // Get enabled proxies
    let proxies = enabled_proxies(&mods);

    // Resolve the agent
    let agent_server = agent
        .resolve()
        .await
        .map_err(|e| sacp::util::internal_error(format!("Failed to resolve agent: {}", e)))?;

    // MCP servers are represented as mods with `ModKind::MCP` in `mods`.
    // Build MCP servers from enabled MCP-type mods so they can be attached to sessions.
    let mcp_sources = enabled_mcp_servers(&mods);
    let mcp_servers = build_mcp_servers(mcp_sources).await?;

    // TODO: Apply trace_dir to conductor when needed

    let agent = AcpAgent::new(agent_server);

    // Build the conductor
    let conductor = ConductorImpl::new_agent(
        "symposium-conductor",
        {
            async move |init_req| {
                tracing::info!(
                    "Building proxy chain with mods: {:?}",
                    proxies.iter().map(|s| s.display_name()).collect::<Vec<_>>()
                );
                let proxies = build_proxies(proxies).await?;
                Ok((init_req, proxies, DynConnectTo::new(agent)))
            }
        },
        McpBridgeMode::default(),
    );

    // Connect to the conductor
    Client
        .builder()
        .on_receive_dispatch(
            async |message_cx: Dispatch, _cx| {
                // Incoming message from the conductor: forward via ConfigAgent to client
                config_agent_tx
                    .unbounded_send(ConfigAgentMessage::MessageToClient(message_cx))
                    .map_err(|_| sacp::util::internal_error("ConfigAgent closed"))
            },
            sacp::on_receive_dispatch!(),
        )
        .connect_with(conductor, async |connection| {
            // Initialize the conductor
            let _init_response = connection
                .send_request(InitializeRequest::new(
                    sacp::schema::ProtocolVersion::LATEST,
                ))
                .block_task()
                .await?;

            while let Some(message) = rx.recv().await {
                match message {
                    ConductorMessage::NewSession {
                        mut request,
                        responder,
                    } => {
                        request.mcp_servers.extend(mcp_servers.clone());

                        let config_agent_tx = config_agent_tx.clone();
                        let self_handle = self_handle.clone();
                        let workspace_path = workspace_path.clone();
                        connection.send_request(request).on_receiving_result(
                            async move |result| {
                                match result {
                                    Ok(response) => {
                                        // Send to ConfigAgent so it can store the session mapping
                                        config_agent_tx
                                            .unbounded_send(ConfigAgentMessage::NewSessionCreated {
                                                response,
                                                conductor: self_handle,
                                                workspace_path,
                                                responder,
                                            })
                                            .map_err(|_| {
                                                sacp::util::internal_error("ConfigAgent closed")
                                            })
                                    }
                                    Err(e) => {
                                        // Forward error directly to client
                                        responder.respond_with_error(e)
                                    }
                                }
                            },
                        )?;
                    }

                    ConductorMessage::Prompt { request, responder } => {
                        if let Err(e) = connection
                            .send_request(request)
                            .forward_response_to(responder)
                        {
                            tracing::error!("Failed to forward prompt to conductor: {}", e);
                        }
                    }

                    ConductorMessage::ForwardMessage { message } => {
                        if let Err(e) = connection.send_proxied_message_to(Agent, message) {
                            tracing::error!("Failed to forward message to conductor: {}", e);
                        }
                    }

                    ConductorMessage::Pause { resume_tx_sender } => {
                        // Create the resume channel
                        let (resume_tx, resume_rx) = oneshot::channel::<()>();

                        // Send the resume_tx back to the caller
                        if resume_tx_sender.send(resume_tx).is_err() {
                            tracing::warn!("Failed to send resume_tx - caller dropped");
                            continue;
                        }

                        // Wait for resume signal (or channel drop)
                        tracing::debug!("Conductor paused, waiting for resume");
                        let _ = resume_rx.await;
                        tracing::debug!("Conductor resumed");
                    }
                }
            }

            tracing::debug!("Conductor actor shutting down");
            Ok(())
        })
        .await
}
