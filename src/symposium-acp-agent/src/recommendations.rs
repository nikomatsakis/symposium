//! Recommendations - what components to suggest for a workspace
//!
//! This module handles recommending agents and extensions based on workspace
//! characteristics. Recommendations are loaded from a built-in TOML file that
//! is embedded in the binary.

use crate::registry::{CargoDistribution, ComponentSource, NpxDistribution};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Built-in recommendations TOML, embedded at compile time
const BUILTIN_RECOMMENDATIONS_TOML: &str = include_str!("builtin_recommendations.toml");

/// A recommendation for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// The source of the component (this IS the identity)
    pub source: ComponentSource,

    /// Human-readable name for display
    #[serde(default)]
    pub name: Option<String>,

    /// Description of what this component does
    #[serde(default)]
    pub description: Option<String>,

    /// Conditions that must be met for this recommendation to apply
    #[serde(default)]
    pub conditions: Vec<Condition>,

    /// Priority for ordering (higher = more important, shown first)
    #[serde(default)]
    pub priority: i32,
}

impl Recommendation {
    /// Get the display name for this recommendation
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.source.display_name())
    }
}

/// Conditions for when a recommendation applies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// File exists in the workspace root
    FileExists { path: String },

    /// File matches a glob pattern anywhere in workspace
    GlobMatches { pattern: String },

    /// Always recommend (unconditional)
    Always,
}

impl Condition {
    /// Check if this condition is met for the given workspace
    pub fn is_met(&self, workspace_path: &Path) -> bool {
        match self {
            Condition::Always => true,
            Condition::FileExists { path } => workspace_path.join(path).exists(),
            Condition::GlobMatches { pattern } => {
                // For now, just check if the pattern file exists at root
                // TODO: Implement proper glob matching
                workspace_path.join(pattern).exists()
            }
        }
    }
}

/// The recommendations file format
#[derive(Debug, Clone, Deserialize)]
struct RecommendationsFile {
    /// Default agent recommendation
    #[serde(default)]
    agent: Option<AgentRecommendation>,

    /// Extension recommendations
    #[serde(default)]
    extensions: Vec<ExtensionRecommendation>,
}

/// Agent recommendation in TOML format
#[derive(Debug, Clone, Deserialize)]
struct AgentRecommendation {
    /// NPX package (shorthand)
    #[serde(default)]
    npx: Option<String>,

    /// Full source specification (alternative to shorthand)
    #[serde(default)]
    source: Option<ComponentSource>,

    #[serde(default)]
    name: Option<String>,

    #[serde(default)]
    description: Option<String>,
}

/// Extension recommendation in TOML format
#[derive(Debug, Clone, Deserialize)]
struct ExtensionRecommendation {
    /// Builtin name (shorthand for ComponentSource::Builtin)
    #[serde(default)]
    builtin: Option<String>,

    /// Cargo crate (shorthand for ComponentSource::Cargo)
    #[serde(default)]
    cargo: Option<CargoShorthand>,

    /// Full source specification (alternative to shorthand)
    #[serde(default)]
    source: Option<ComponentSource>,

    #[serde(default)]
    name: Option<String>,

    #[serde(default)]
    description: Option<String>,

    #[serde(default)]
    conditions: Vec<Condition>,

    #[serde(default)]
    priority: i32,
}

/// Shorthand for cargo crate specification
#[derive(Debug, Clone, Deserialize)]
struct CargoShorthand {
    #[serde(rename = "crate")]
    crate_name: String,
    #[serde(default)]
    args: Vec<String>,
}

/// Loaded recommendations
#[derive(Debug, Clone)]
pub struct Recommendations {
    /// The recommended agent
    pub agent: Option<Recommendation>,

    /// All extension recommendations
    pub extensions: Vec<Recommendation>,
}

impl Recommendations {
    /// Load the built-in recommendations
    pub fn load_builtin() -> Result<Self> {
        Self::from_toml(BUILTIN_RECOMMENDATIONS_TOML)
    }

    /// Parse recommendations from TOML string
    fn from_toml(toml_str: &str) -> Result<Self> {
        let file: RecommendationsFile =
            toml::from_str(toml_str).context("Failed to parse recommendations TOML")?;

        // Convert agent recommendation
        let agent = file.agent.map(|a| {
            let source = if let Some(npx) = a.npx {
                ComponentSource::Npx(NpxDistribution {
                    package: npx,
                    args: vec![],
                    env: BTreeMap::new(),
                })
            } else if let Some(source) = a.source {
                source
            } else {
                // Default to Claude Code
                ComponentSource::Npx(NpxDistribution {
                    package: "@zed-industries/claude-code-acp@latest".to_string(),
                    args: vec![],
                    env: BTreeMap::new(),
                })
            };

            Recommendation {
                source,
                name: a.name,
                description: a.description,
                conditions: vec![Condition::Always],
                priority: 100, // Agent has highest priority
            }
        });

        // Convert extension recommendations
        let extensions = file
            .extensions
            .into_iter()
            .map(|e| {
                let source = if let Some(builtin) = e.builtin {
                    ComponentSource::Builtin(builtin)
                } else if let Some(cargo) = e.cargo {
                    ComponentSource::Cargo(CargoDistribution {
                        crate_name: cargo.crate_name,
                        version: None,
                        binary: None,
                        args: cargo.args,
                    })
                } else if let Some(source) = e.source {
                    source
                } else {
                    // This shouldn't happen with valid TOML
                    ComponentSource::Builtin("unknown".to_string())
                };

                Recommendation {
                    source,
                    name: e.name,
                    description: e.description,
                    conditions: if e.conditions.is_empty() {
                        vec![Condition::Always]
                    } else {
                        e.conditions
                    },
                    priority: e.priority,
                }
            })
            .collect();

        Ok(Self { agent, extensions })
    }

    /// Get recommendations that apply to a specific workspace
    pub fn for_workspace(&self, workspace_path: &Path) -> WorkspaceRecommendations {
        let agent = self.agent.clone();

        let mut extensions: Vec<Recommendation> = self
            .extensions
            .iter()
            .filter(|r| r.conditions.iter().all(|c| c.is_met(workspace_path)))
            .cloned()
            .collect();

        // Sort by priority (highest first)
        extensions.sort_by(|a, b| b.priority.cmp(&a.priority));

        WorkspaceRecommendations { agent, extensions }
    }
}

/// Recommendations filtered for a specific workspace
#[derive(Debug, Clone)]
pub struct WorkspaceRecommendations {
    pub agent: Option<Recommendation>,
    pub extensions: Vec<Recommendation>,
}

impl WorkspaceRecommendations {
    /// Get all extension sources as a set (for diffing with config)
    pub fn extension_sources(&self) -> Vec<ComponentSource> {
        self.extensions.iter().map(|r| r.source.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_builtin_recommendations() {
        let recs = Recommendations::load_builtin().expect("Should load builtin recommendations");

        // Should have an agent recommendation
        assert!(recs.agent.is_some(), "Should have agent recommendation");

        // Should have some extension recommendations
        assert!(
            !recs.extensions.is_empty(),
            "Should have extension recommendations"
        );
    }

    #[test]
    fn test_builtin_agent_is_claude_code() {
        let recs = Recommendations::load_builtin().unwrap();
        let agent = recs.agent.unwrap();

        // Should recommend Claude Code via NPX
        match &agent.source {
            ComponentSource::Npx(npx) => {
                assert!(
                    npx.package.contains("claude-code"),
                    "Default agent should be Claude Code"
                );
            }
            _ => panic!("Expected NPX source for agent"),
        }
    }

    #[test]
    fn test_workspace_filtering() {
        let toml = r#"
[[extensions]]
builtin = "always-on"
name = "Always On"

[[extensions]]
builtin = "rust-only"
name = "Rust Only"
[[extensions.conditions]]
type = "file_exists"
path = "Cargo.toml"
"#;

        let recs = Recommendations::from_toml(toml).unwrap();

        // Create a temp directory without Cargo.toml
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());

        // Should only have the "always-on" extension
        assert_eq!(workspace_recs.extensions.len(), 1);
        assert_eq!(workspace_recs.extensions[0].display_name(), "Always On");

        // Now create Cargo.toml
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());

        // Should have both extensions
        assert_eq!(workspace_recs.extensions.len(), 2);
    }

    #[test]
    fn test_priority_ordering() {
        let toml = r#"
[[extensions]]
builtin = "low"
priority = 10

[[extensions]]
builtin = "high"
priority = 100

[[extensions]]
builtin = "medium"
priority = 50
"#;

        let recs = Recommendations::from_toml(toml).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());

        // Should be ordered by priority (high, medium, low)
        assert_eq!(
            workspace_recs.extensions[0].source,
            ComponentSource::Builtin("high".to_string())
        );
        assert_eq!(
            workspace_recs.extensions[1].source,
            ComponentSource::Builtin("medium".to_string())
        );
        assert_eq!(
            workspace_recs.extensions[2].source,
            ComponentSource::Builtin("low".to_string())
        );
    }
}
