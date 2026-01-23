//! Config mode actor - handles the interactive configuration "phone tree" UI.
//!
//! This actor is spawned when a user enters config mode via `/symposium:config`.
//! It owns the configuration state and processes user input through a simple
//! text-based menu system.

use super::ConfigAgentMessage;
use crate::recommendations::WorkspaceRecommendations;
use crate::registry::{list_agents_with_sources, ComponentSource};
use crate::user_config::{GlobalAgentConfig, WorkspaceConfig};
use futures::channel::mpsc::{self, UnboundedSender};
use futures::StreamExt;
use regex::Regex;
use sacp::link::AgentToClient;
use sacp::schema::SessionId;
use sacp::JrConnectionCx;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::sync::oneshot;

/// Result of handling menu input.
enum MenuAction {
    /// Exit the menu loop (save or cancel was chosen).
    Done,
    /// Redisplay the menu (state changed).
    Redisplay,
    /// Just wait for more input (invalid command, no state change).
    Continue,
}

/// Messages sent to the config mode actor.
pub enum ConfigModeInput {
    /// User sent a prompt (the text content).
    UserInput(String),
}

/// Messages sent from the config mode actor back to ConfigAgent.
pub enum ConfigModeOutput {
    /// Send this text to the user.
    SendMessage(String),

    /// Configuration is complete - save and exit.
    Done {
        /// The final configuration to save.
        config: WorkspaceConfig,
    },

    /// User cancelled - exit without saving.
    Cancelled,
}

/// Handle to communicate with the config mode actor.
#[derive(Clone)]
pub struct ConfigModeHandle {
    tx: mpsc::Sender<ConfigModeInput>,
}

impl ConfigModeHandle {
    /// Spawn a new config mode actor.
    ///
    /// Returns a handle for sending input to the actor.
    ///
    /// If `config` is None, this is initial setup - the actor will use
    /// recommendations to create the initial configuration.
    ///
    /// The `resume_tx` is an optional oneshot sender that, when dropped, will
    /// signal the conductor to resume processing. If provided, it will be
    /// dropped when the actor exits (either save or cancel).
    ///
    /// The `default_agent_override` is used for testing - if Some, it bypasses
    /// the GlobalAgentConfig::load() and uses this agent for initial setup.
    pub fn spawn(
        config: Option<WorkspaceConfig>,
        workspace_path: PathBuf,
        recommendations: Option<WorkspaceRecommendations>,
        default_agent_override: Option<ComponentSource>,
        session_id: SessionId,
        config_agent_tx: UnboundedSender<ConfigAgentMessage>,
        resume_tx: Option<oneshot::Sender<()>>,
        cx: &JrConnectionCx<AgentToClient>,
    ) -> Result<Self, sacp::Error> {
        let (tx, rx) = mpsc::channel(32);
        let handle = Self { tx };

        let actor = ConfigModeActor {
            config,
            workspace_path,
            recommendations,
            default_agent_override,
            session_id,
            config_agent_tx,
            rx,
            _resume_tx: resume_tx,
        };

        cx.spawn(actor.run())?;

        Ok(handle)
    }

    /// Send user input to the actor.
    pub async fn send_input(&self, text: String) -> Result<(), sacp::Error> {
        self.tx
            .clone()
            .try_send(ConfigModeInput::UserInput(text))
            .map_err(|_| sacp::util::internal_error("Config mode actor closed"))
    }
}

/// The config mode actor state.
struct ConfigModeActor {
    /// Current configuration. None means initial setup (no config exists yet).
    config: Option<WorkspaceConfig>,
    /// The workspace this configuration is for.
    workspace_path: PathBuf,
    /// Recommendations for this workspace.
    recommendations: Option<WorkspaceRecommendations>,
    /// Override for the global agent config. If Some, bypasses GlobalAgentConfig::load().
    /// Used for testing.
    default_agent_override: Option<ComponentSource>,
    session_id: SessionId,
    config_agent_tx: UnboundedSender<ConfigAgentMessage>,
    rx: mpsc::Receiver<ConfigModeInput>,
    /// When dropped, signals the conductor to resume. We never send to this,
    /// just hold it until the actor exits.
    _resume_tx: Option<oneshot::Sender<()>>,
}

impl ConfigModeActor {
    /// Main entry point - runs the actor.
    async fn run(mut self) -> Result<(), sacp::Error> {
        // If no config exists (initial setup), we need to set up
        let mut config = match self.config.take() {
            Some(config) => config,
            None => {
                self.send_message("Welcome to Symposium!\n\n");

                // Check for global agent config (or use override for testing)
                let global_agent = if let Some(agent) = self.default_agent_override.take() {
                    Some(agent)
                } else {
                    match GlobalAgentConfig::load() {
                        Ok(Some(global)) => Some(global.agent),
                        Ok(None) => None,
                        Err(e) => {
                            tracing::warn!("Failed to load global agent config: {}", e);
                            None
                        }
                    }
                };

                let agent = match global_agent {
                    Some(agent) => {
                        self.send_message(&format!(
                            "Using your default agent: **{}**\n\n",
                            agent.display_name()
                        ));
                        agent
                    }
                    None => {
                        // No global agent - need to select one
                        self.send_message("No default agent configured. Let's choose one.\n\n");
                        match self.select_agent().await {
                            Some(agent) => {
                                // Save as global default
                                if let Err(e) = GlobalAgentConfig::new(agent.clone()).save() {
                                    tracing::warn!("Failed to save global agent config: {}", e);
                                }
                                agent
                            }
                            None => {
                                self.send_message("Agent selection cancelled.\n");
                                self.cancelled();
                                return Ok(());
                            }
                        }
                    }
                };

                // Create config with selected agent and recommended extensions
                let extensions = self
                    .recommendations
                    .as_ref()
                    .map(|r| r.extension_sources())
                    .unwrap_or_default();

                self.send_message("Configuration created with recommended extensions.\n\n");
                WorkspaceConfig::new(agent, extensions)
            }
        };

        self.main_menu_loop(&mut config).await;

        Ok(())
    }

    /// Prompt user to select an agent from the registry.
    /// Returns None if cancelled or an error occurred.
    async fn select_agent(&mut self) -> Option<ComponentSource> {
        self.send_message("Fetching available agents...\n");

        let agents = match list_agents_with_sources().await {
            Ok(agents) => agents,
            Err(e) => {
                self.send_message(&format!("Failed to fetch agents: {}\n", e));
                return None;
            }
        };

        if agents.is_empty() {
            self.send_message("No agents available.\n");
            return None;
        }

        // Show the list
        let mut msg = String::new();
        msg.push_str("# Select an Agent\n\n");
        for (i, (entry, _)) in agents.iter().enumerate() {
            msg.push_str(&format!("{}. {}\n", i + 1, entry.name));
        }
        msg.push_str("\nEnter a number to select, or `cancel` to abort:\n");
        self.send_message(msg);

        // Wait for selection
        loop {
            let Some(input) = self.next_input().await else {
                return None;
            };
            let input = input.trim();

            if input.eq_ignore_ascii_case("cancel") {
                return None;
            }

            if let Ok(idx) = input.parse::<usize>() {
                if idx >= 1 && idx <= agents.len() {
                    let (entry, source) = &agents[idx - 1];
                    self.send_message(&format!("Selected: **{}**\n\n", entry.name));
                    return Some(source.clone());
                }
            }

            self.send_message(&format!(
                "Invalid selection. Please enter 1-{} or `cancel`.\n",
                agents.len()
            ));
        }
    }

    /// Create initial configuration from recommendations (legacy, now unused).
    #[allow(dead_code)]
    fn create_initial_config(&self) -> Option<WorkspaceConfig> {
        let recs = self.recommendations.as_ref()?;

        // Get the recommended agent
        let agent = recs.agent.as_ref()?.source.clone();

        // Get recommended extensions
        let extensions: Vec<ComponentSource> = recs.extension_sources();

        Some(WorkspaceConfig::new(agent, extensions))
    }

    /// Get ordered list of extensions with their sources for display.
    fn get_extension_list(&self, config: &WorkspaceConfig) -> Vec<(ComponentSource, bool)> {
        config
            .extensions
            .iter()
            .filter_map(|(key, ext_config)| {
                ComponentSource::from_config_key(key)
                    .ok()
                    .map(|source| (source, ext_config.enabled))
            })
            .collect()
    }

    /// Wait for the next user input.
    async fn next_input(&mut self) -> Option<String> {
        match self.rx.next().await {
            Some(ConfigModeInput::UserInput(text)) => Some(text),
            None => None,
        }
    }

    /// Send a message to the user.
    fn send_message(&self, text: impl Into<String>) {
        self.config_agent_tx
            .unbounded_send(ConfigAgentMessage::ConfigModeOutput(
                self.session_id.clone(),
                ConfigModeOutput::SendMessage(text.into()),
            ))
            .ok();
    }

    /// Signal that configuration is done (save and exit).
    fn done(&self, config: &WorkspaceConfig) {
        self.config_agent_tx
            .unbounded_send(ConfigAgentMessage::ConfigModeOutput(
                self.session_id.clone(),
                ConfigModeOutput::Done {
                    config: config.clone(),
                },
            ))
            .ok();
    }

    /// Signal that configuration was cancelled.
    fn cancelled(&self) {
        self.config_agent_tx
            .unbounded_send(ConfigAgentMessage::ConfigModeOutput(
                self.session_id.clone(),
                ConfigModeOutput::Cancelled,
            ))
            .ok();
    }

    /// Main menu loop.
    async fn main_menu_loop(&mut self, config: &mut WorkspaceConfig) {
        self.show_main_menu(config);

        loop {
            let Some(input) = self.next_input().await else {
                return;
            };

            match self.handle_main_menu_input(&input, config).await {
                MenuAction::Done => return,
                MenuAction::Redisplay => self.show_main_menu(config),
                MenuAction::Continue => {}
            }
        }
    }

    /// Handle input in the main menu.
    async fn handle_main_menu_input(
        &mut self,
        text: &str,
        config: &mut WorkspaceConfig,
    ) -> MenuAction {
        let text = text.trim();
        let text_upper = text.to_uppercase();

        // Save and exit
        if text_upper == "SAVE" {
            self.done(config);
            return MenuAction::Done;
        }

        // Cancel without saving
        if text_upper == "CANCEL" {
            self.cancelled();
            return MenuAction::Done;
        }

        // Change agent
        if text_upper == "A" || text_upper == "AGENT" {
            if let Some(new_agent) = self.select_agent().await {
                config.agent = new_agent.clone();
                // Also update global agent config
                if let Err(e) = GlobalAgentConfig::new(new_agent.clone()).save() {
                    tracing::warn!("Failed to save global agent config: {}", e);
                    self.send_message(&format!(
                        "Note: Could not save as default agent: {}\n",
                        e
                    ));
                } else {
                    self.send_message("Updated default agent for future workspaces.\n");
                }
                return MenuAction::Redisplay;
            }
            // Selection was cancelled, just redisplay menu
            return MenuAction::Redisplay;
        }

        let extensions = self.get_extension_list(config);

        // Toggle extension by index (1-based)
        if let Ok(display_index) = text.parse::<usize>() {
            if display_index >= 1 && display_index <= extensions.len() {
                let (source, _enabled) = &extensions[display_index - 1];
                let new_enabled = config.toggle_extension(source);
                let status = if new_enabled { "enabled" } else { "disabled" };
                self.send_message(format!(
                    "Extension `{}` is now {}.",
                    source.display_name(),
                    status
                ));
                return MenuAction::Redisplay;
            } else if extensions.is_empty() {
                self.send_message("No extensions configured.");
                return MenuAction::Continue;
            } else {
                self.send_message(format!(
                    "Invalid index. Please enter 1-{}.",
                    extensions.len()
                ));
                return MenuAction::Continue;
            }
        }

        // Move command: "move X to Y" or "move X to start/end" (1-based)
        // Note: Since we use BTreeMap, ordering is by key, not insertion order.
        // For now, we don't support reordering - could add a priority field later.
        static MOVE_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?i)^move\s+(\d+)\s+to\s+(\d+|start|end)$").unwrap());

        if MOVE_RE.captures(text).is_some() {
            self.send_message(
                "Extension reordering is not yet supported with the new config format.",
            );
            return MenuAction::Continue;
        }

        // Unknown command
        self.send_message(format!("Unknown command: `{}`", text));
        MenuAction::Continue
    }

    /// Show the main menu.
    fn show_main_menu(&self, config: &WorkspaceConfig) {
        let mut msg = String::new();
        msg.push_str("# Configuration\n\n");
        msg.push_str(&format!(
            "Workspace: `{}`\n\n",
            self.workspace_path.display()
        ));

        // Current agent
        msg.push_str(&format!("* **Agent:** {}\n", config.agent.display_name()));

        // Extensions
        let extensions = self.get_extension_list(config);
        msg.push_str("* **Extensions:**\n");
        if extensions.is_empty() {
            msg.push_str("    * (none configured)\n");
        } else {
            for (i, (source, enabled)) in extensions.iter().enumerate() {
                let display_index = i + 1;
                let name = source.display_name();
                if *enabled {
                    msg.push_str(&format!("    {}. {}\n", display_index, name));
                } else {
                    msg.push_str(&format!("    {}. ~~{}~~ (disabled)\n", display_index, name));
                }
            }
        }
        msg.push('\n');

        // Commands
        msg.push_str("# Commands\n\n");
        msg.push_str("- `a` - Change agent\n");
        if !extensions.is_empty() {
            msg.push_str("- `1`, `2`, ... - Toggle extension enabled/disabled\n");
        }
        msg.push_str("- `save` - Save for future sessions\n");
        msg.push_str("- `cancel` - Exit without saving\n");

        self.send_message(msg);
    }
}
