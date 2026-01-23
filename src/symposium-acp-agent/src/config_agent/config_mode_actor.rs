//! Config mode actor - handles the interactive configuration "phone tree" UI.
//!
//! This actor is spawned when a user enters config mode via `/symposium:config`.
//! It owns the configuration state and processes user input through a simple
//! text-based menu system.

use super::ConfigAgentMessage;
use crate::recommendations::WorkspaceRecommendations;
use crate::registry::ComponentSource;
use crate::user_config::WorkspaceConfig;
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
    pub fn spawn(
        config: Option<WorkspaceConfig>,
        workspace_path: PathBuf,
        recommendations: Option<WorkspaceRecommendations>,
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
        // If no config exists (initial setup), create from recommendations
        let mut config = match self.config.take() {
            Some(config) => config,
            None => {
                self.send_message(
                    "Welcome to Symposium!\n\n\
                     No configuration found. Setting up your workspace.\n",
                );
                match self.create_initial_config() {
                    Some(config) => {
                        self.send_message("Created configuration from recommendations.\n");
                        config
                    }
                    None => {
                        self.send_message(
                            "No recommendations available. Cannot create configuration.\n",
                        );
                        self.cancelled();
                        return Ok(());
                    }
                }
            }
        };

        self.main_menu_loop(&mut config).await;

        Ok(())
    }

    /// Create initial configuration from recommendations.
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
        if !extensions.is_empty() {
            msg.push_str("- `1`, `2`, ... - Toggle extension enabled/disabled\n");
        }
        msg.push_str("- `save` - Save for future sessions\n");
        msg.push_str("- `cancel` - Exit without saving\n");

        self.send_message(msg);
    }
}
