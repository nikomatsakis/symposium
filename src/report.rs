//! Structured sync report layer.
//!
//! Emits user-facing events during `cargo agents sync` as tracing events
//! carrying a single `report` field whose value is a serialized
//! `SyncReportEvent`. A custom tracing layer picks these up and either
//! pretty-prints them (`--verbose`) or accumulates JSON (`--json`).

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tracing::field::{Field, Visit};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// The output mode for the report layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportMode {
    /// Pretty-print events to stderr as they arrive.
    Verbose,
    /// Accumulate events, emit as a JSON array at the end.
    Json,
}

/// A structured event emitted during sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncReportEvent {
    /// A plugin was considered and either matched or was skipped.
    PluginConsidered {
        plugin: String,
        matched: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A skill group within a plugin was considered.
    SkillGroupConsidered {
        plugin: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        group_crates: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        matched: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        skills_found: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A directory was searched for SKILL.md files.
    SkillSourceSearched {
        plugin: String,
        source: String,
        path: String,
        skills_found: usize,
    },

    /// An individual skill was evaluated.
    SkillConsidered {
        skill: String,
        plugin: String,
        matched: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// A skill was installed to an agent's directory.
    SkillInstalled {
        skill: String,
        agent: String,
        dest: String,
    },

    /// A stale skill directory was removed.
    SkillRemoved { path: String },

    /// A hook was registered for an agent.
    HookRegistered { agent: String, hook: String },

    /// An MCP server was registered for an agent.
    McpServerRegistered { agent: String, server: String },

    /// Summary line at the end.
    SyncSummary {
        workspace_deps: usize,
        plugins_considered: usize,
        skills_installed: usize,
        skills_removed: usize,
    },
}

impl std::fmt::Display for SyncReportEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}

impl SyncReportEvent {
    fn format_human(&self) -> String {
        match self {
            Self::PluginConsidered {
                plugin,
                matched,
                reason,
            } => {
                if *matched {
                    format!("  plugin {plugin}: matched")
                } else {
                    let r = reason.as_deref().unwrap_or("predicates not satisfied");
                    format!("  plugin {plugin}: skipped ({r})")
                }
            }
            Self::SkillGroupConsidered {
                plugin,
                group_crates,
                source,
                matched,
                skills_found,
                reason,
            } => {
                let crates_str = group_crates.as_deref().unwrap_or("*");
                let source_str = source.as_deref().unwrap_or("unknown");
                if *matched {
                    let count = skills_found.unwrap_or(0);
                    format!(
                        "    group [{crates_str}] in {plugin}: matched, source={source_str}, {count} skill(s) found"
                    )
                } else {
                    let r = reason.as_deref().unwrap_or("predicates not satisfied");
                    format!("    group [{crates_str}] in {plugin}: skipped ({r})")
                }
            }
            Self::SkillSourceSearched {
                plugin,
                source,
                path,
                skills_found,
            } => {
                format!(
                    "      searched {source} ({plugin}): {path} → {skills_found} skill(s)"
                )
            }
            Self::SkillConsidered {
                skill,
                plugin,
                matched,
                reason,
            } => {
                if *matched {
                    format!("      skill {skill} ({plugin}): included")
                } else {
                    let r = reason.as_deref().unwrap_or("predicates not satisfied");
                    format!("      skill {skill} ({plugin}): skipped ({r})")
                }
            }
            Self::SkillInstalled { skill, agent, dest } => {
                format!("  installed {skill} for {agent} → {dest}")
            }
            Self::SkillRemoved { path } => {
                format!("  removed stale {path}")
            }
            Self::HookRegistered { agent, hook } => {
                format!("  registered hook {hook} for {agent}")
            }
            Self::McpServerRegistered { agent, server } => {
                format!("  registered MCP server {server} for {agent}")
            }
            Self::SyncSummary {
                workspace_deps,
                plugins_considered,
                skills_installed,
                skills_removed,
            } => {
                format!(
                    "sync complete: {workspace_deps} deps, {plugins_considered} plugins considered, \
                     {skills_installed} skills installed, {skills_removed} removed"
                )
            }
        }
    }
}

/// Handle returned when creating a report layer, allowing the caller
/// to drain accumulated JSON after the operation completes.
#[derive(Clone)]
pub struct ReportHandle {
    buffer: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl ReportHandle {
    /// Drain accumulated JSON events. Only meaningful in `Json` mode.
    pub fn drain(&self) -> Vec<serde_json::Value> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }
}

/// Tracing layer that captures events with a `report` field.
pub struct ReportLayer {
    mode: ReportMode,
    buffer: Arc<Mutex<Vec<serde_json::Value>>>,
    max_level: tracing::Level,
}

impl ReportLayer {
    pub fn new(mode: ReportMode, max_level: tracing::Level) -> (Self, ReportHandle) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let handle = ReportHandle {
            buffer: buffer.clone(),
        };
        (
            Self {
                mode,
                buffer,
                max_level,
            },
            handle,
        )
    }
}

struct ReportVisitor {
    report_json: Option<String>,
}

impl Visit for ReportVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "report" {
            self.report_json = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "report" {
            self.report_json = Some(format!("{value:?}"));
        }
    }
}

impl<S> Layer<S> for ReportLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().level() > &self.max_level {
            return;
        }

        let mut visitor = ReportVisitor { report_json: None };
        event.record(&mut visitor);

        let Some(json_str) = visitor.report_json else {
            return;
        };

        match self.mode {
            ReportMode::Verbose => {
                if let Ok(evt) = serde_json::from_str::<SyncReportEvent>(&json_str) {
                    eprintln!("{}", evt.format_human());
                }
            }
            ReportMode::Json => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    self.buffer.lock().unwrap().push(val);
                }
            }
        }
    }
}
