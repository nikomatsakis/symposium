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

    /// Explain why this recommendation should be added (for new recommendations)
    pub fn explain_why_added(&self) -> Vec<String> {
        self.conditions
            .iter()
            .filter_map(|c| c.explain_why_added())
            .collect()
    }

    /// Explain why this recommendation is stale (for removed recommendations)
    pub fn explain_why_stale(&self) -> Vec<String> {
        self.conditions
            .iter()
            .filter_map(|c| c.explain_why_stale())
            .collect()
    }

    /// Format explanation for display (joins all reasons)
    pub fn format_added_explanation(&self) -> String {
        let reasons = self.explain_why_added();
        if reasons.is_empty() {
            String::new()
        } else {
            format!("[{}]", reasons.join(", "))
        }
    }

    /// Format stale explanation for display (joins all reasons)
    pub fn format_stale_explanation(&self) -> String {
        let reasons = self.explain_why_stale();
        if reasons.is_empty() {
            String::new()
        } else {
            format!("[{}]", reasons.join(", "))
        }
    }
}

/// Conditions for when a recommendation applies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Explain why this condition causes a recommendation to be added
    pub fn explain_why_added(&self) -> Option<String> {
        match self {
            Condition::Always => None,
            Condition::FileExists { path } => Some(format!("because a file `{path}` exists")),
            Condition::GlobMatches { pattern } => {
                Some(format!("because files matching `{pattern}` exist"))
            }
        }
    }

    /// Explain why this condition causes a recommendation to be stale
    pub fn explain_why_stale(&self) -> Option<String> {
        match self {
            Condition::Always => None,
            Condition::FileExists { path } => Some(format!("because no file `{path}` exists")),
            Condition::GlobMatches { pattern } => {
                Some(format!("because no files matching `{pattern}` exist"))
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

    /// Get a recommendation by its source
    pub fn get_recommendation(&self, source: &ComponentSource) -> Option<&Recommendation> {
        self.extensions.iter().find(|r| &r.source == source)
    }
}

// ============================================================================
// Recommendation Diffing
// ============================================================================

use crate::user_config::WorkspaceConfig;
use std::collections::HashSet;

/// A new recommendation that isn't in the user's config yet
#[derive(Debug, Clone)]
pub struct NewRecommendation {
    /// The recommendation
    pub recommendation: Recommendation,
    /// Whether the user wants to accept it (toggled via UI)
    pub accepted: bool,
}

impl NewRecommendation {
    /// Format this recommendation for display
    pub fn format_display(&self, index: usize) -> String {
        let action = if self.accepted { "ADD" } else { "IGNORE" };
        let name = self.recommendation.display_name();
        let explanation = if self.accepted {
            self.recommendation.format_added_explanation()
        } else {
            // When ignoring, flip the explanation to "even though..."
            let reasons = self.recommendation.explain_why_added();
            if reasons.is_empty() {
                String::new()
            } else {
                format!("[even though {}]", reasons.join(", ").trim_start_matches("because "))
            }
        };

        if explanation.is_empty() {
            format!("{}. {} {}", index, action, name)
        } else {
            format!("{}. {} {} {}", index, action, name, explanation)
        }
    }
}

/// A stale extension that's in config but no longer recommended
#[derive(Debug, Clone)]
pub struct StaleExtension {
    /// The component source
    pub source: ComponentSource,
    /// The conditions that originally caused this to be recommended
    pub conditions: Vec<Condition>,
}

impl StaleExtension {
    /// Format this stale extension for display
    pub fn format_display(&self) -> String {
        let name = self.source.display_name();
        let reasons: Vec<String> = self
            .conditions
            .iter()
            .filter_map(|c| c.explain_why_stale())
            .collect();

        if reasons.is_empty() {
            format!("- {} is stale", name)
        } else {
            format!("- {} is stale [{}]", name, reasons.join(", "))
        }
    }
}

/// The diff between recommendations and current config
#[derive(Debug, Clone)]
pub struct RecommendationDiff {
    /// New recommendations (recommended but not in config)
    pub new: Vec<NewRecommendation>,
    /// Stale extensions (in config but no longer recommended)
    pub stale: Vec<StaleExtension>,
}

impl RecommendationDiff {
    /// Compute the diff between recommendations and config
    pub fn compute(recommendations: &WorkspaceRecommendations, config: &WorkspaceConfig) -> Self {
        // Get the set of recommended sources
        let recommended_sources: HashSet<_> = recommendations
            .extensions
            .iter()
            .map(|r| r.source.to_config_key())
            .collect();

        // Get the set of configured sources
        let configured_sources: HashSet<_> = config.extensions.keys().cloned().collect();

        // New = recommended but not configured
        let new: Vec<NewRecommendation> = recommendations
            .extensions
            .iter()
            .filter(|r| !configured_sources.contains(&r.source.to_config_key()))
            .map(|r| NewRecommendation {
                recommendation: r.clone(),
                accepted: true, // Default to accepted
            })
            .collect();

        // Stale = configured but not recommended
        let stale: Vec<StaleExtension> = config
            .extensions
            .keys()
            .filter(|key| !recommended_sources.contains(*key))
            .filter_map(|key| {
                let source = ComponentSource::from_config_key(key).ok()?;
                let conditions = config.extension_conditions(&source);
                Some(StaleExtension { source, conditions })
            })
            .collect();

        Self { new, stale }
    }

    /// Check if there are any changes to present
    pub fn has_changes(&self) -> bool {
        !self.new.is_empty() || !self.stale.is_empty()
    }

    /// Check if there are only stale items (no new recommendations)
    pub fn is_stale_only(&self) -> bool {
        self.new.is_empty() && !self.stale.is_empty()
    }

    /// Check if there are only new items (no stale)
    pub fn is_new_only(&self) -> bool {
        !self.new.is_empty() && self.stale.is_empty()
    }

    /// Toggle the acceptance of a new recommendation by index (1-based)
    pub fn toggle(&mut self, index: usize) -> Result<(), String> {
        if index == 0 || index > self.new.len() {
            return Err(format!(
                "Invalid index {}. Valid range is 1-{}",
                index,
                self.new.len()
            ));
        }
        self.new[index - 1].accepted = !self.new[index - 1].accepted;
        Ok(())
    }

    /// Format the full prompt for display
    pub fn format_prompt(&self) -> String {
        let mut lines = Vec::new();

        // New recommendations section
        if !self.new.is_empty() {
            lines.push("The following agent extensions are recommended:".to_string());
            lines.push(String::new());
            for (i, rec) in self.new.iter().enumerate() {
                lines.push(rec.format_display(i + 1));
            }
        }

        // Stale section
        if !self.stale.is_empty() {
            if !self.new.is_empty() {
                lines.push(String::new());
            }
            lines.push("The following agent extensions are stale and will be removed:".to_string());
            lines.push(String::new());
            for stale in &self.stale {
                lines.push(stale.format_display());
            }
        }

        // Instructions
        lines.push(String::new());
        lines.push("How would you like to proceed?".to_string());
        lines.push(String::new());

        if self.is_stale_only() {
            // Simplified prompt for stale-only case
            lines.push(
                "Press ENTER to continue, or say LATER to leave your extensions unchanged until next session."
                    .to_string(),
            );
        } else {
            lines.push("* SAVE the new recommendations".to_string());
            lines.push(
                "* IGNORE the new recommendations, you can always add them later".to_string(),
            );
            lines.push("* 1...N toggle the status of a specific recommendation".to_string());
            lines.push(String::new());
            lines.push(
                "Or you can say LATER to leave your extensions unchanged. You will get this prompt at the start of the next session."
                    .to_string(),
            );
        }

        lines.join("\n")
    }

    /// Apply the "SAVE" action to the config
    /// - Adds new recommendations with their accepted state (enabled = accepted)
    /// - Removes stale extensions
    pub fn apply_save(&self, config: &mut WorkspaceConfig) {
        // Add new recommendations
        for rec in &self.new {
            config.add_extension_with_conditions(
                rec.recommendation.source.clone(),
                rec.accepted, // enabled = accepted
                rec.recommendation.conditions.clone(),
            );
        }

        // Remove stale extensions
        for stale in &self.stale {
            config.remove_extension(&stale.source);
        }
    }

    /// Apply the "IGNORE" action to the config
    /// - Adds new recommendations as disabled (so they won't be asked again)
    /// - Removes stale extensions
    pub fn apply_ignore(&self, config: &mut WorkspaceConfig) {
        // Add new recommendations as disabled
        for rec in &self.new {
            config.add_extension_with_conditions(
                rec.recommendation.source.clone(),
                false, // Always disabled for IGNORE
                rec.recommendation.conditions.clone(),
            );
        }

        // Remove stale extensions
        for stale in &self.stale {
            config.remove_extension(&stale.source);
        }
    }

    /// Apply the stale-only "ENTER" action (just remove stale, no new to add)
    pub fn apply_stale_removal(&self, config: &mut WorkspaceConfig) {
        for stale in &self.stale {
            config.remove_extension(&stale.source);
        }
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

    // ========================================================================
    // Diff tests
    // ========================================================================

    fn make_workspace_recs(extensions: Vec<(&str, Vec<Condition>)>) -> WorkspaceRecommendations {
        WorkspaceRecommendations {
            agent: None,
            extensions: extensions
                .into_iter()
                .map(|(name, conditions)| Recommendation {
                    source: ComponentSource::Builtin(name.to_string()),
                    name: None,
                    description: None,
                    conditions,
                    priority: 0,
                })
                .collect(),
        }
    }

    #[test]
    fn test_diff_new_recommendations() {
        let recs = make_workspace_recs(vec![
            ("foo", vec![Condition::FileExists { path: "Cargo.toml".to_string() }]),
            ("bar", vec![Condition::Always]),
        ]);
        let config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![], // Empty config
        );

        let diff = RecommendationDiff::compute(&recs, &config);

        assert_eq!(diff.new.len(), 2);
        assert!(diff.stale.is_empty());
        assert!(diff.has_changes());
        assert!(diff.is_new_only());
    }

    #[test]
    fn test_diff_stale_extensions() {
        let recs = make_workspace_recs(vec![]); // No recommendations
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        // Add an extension that's not recommended
        config.add_extension_with_conditions(
            ComponentSource::Builtin("old-ext".to_string()),
            true,
            vec![Condition::FileExists { path: "old.txt".to_string() }],
        );

        let diff = RecommendationDiff::compute(&recs, &config);

        assert!(diff.new.is_empty());
        assert_eq!(diff.stale.len(), 1);
        assert_eq!(diff.stale[0].source, ComponentSource::Builtin("old-ext".to_string()));
        assert!(diff.has_changes());
        assert!(diff.is_stale_only());
    }

    #[test]
    fn test_diff_no_changes_when_in_sync() {
        let recs = make_workspace_recs(vec![
            ("foo", vec![Condition::Always]),
        ]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        // Add the same extension that's recommended
        config.add_extension_with_conditions(
            ComponentSource::Builtin("foo".to_string()),
            true,
            vec![Condition::Always],
        );

        let diff = RecommendationDiff::compute(&recs, &config);

        assert!(diff.new.is_empty());
        assert!(diff.stale.is_empty());
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_diff_disabled_extension_not_new() {
        // If an extension is in config but disabled, it's still "known" - not new
        let recs = make_workspace_recs(vec![
            ("foo", vec![Condition::Always]),
        ]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        config.add_extension_with_conditions(
            ComponentSource::Builtin("foo".to_string()),
            false, // Disabled
            vec![Condition::Always],
        );

        let diff = RecommendationDiff::compute(&recs, &config);

        // foo is not new because it's already in config (even though disabled)
        assert!(diff.new.is_empty());
        assert!(diff.stale.is_empty());
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_diff_toggle() {
        let recs = make_workspace_recs(vec![
            ("foo", vec![Condition::Always]),
            ("bar", vec![Condition::Always]),
        ]);
        let config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );

        let mut diff = RecommendationDiff::compute(&recs, &config);

        // Both start as accepted
        assert!(diff.new[0].accepted);
        assert!(diff.new[1].accepted);

        // Toggle first one
        diff.toggle(1).unwrap();
        assert!(!diff.new[0].accepted);
        assert!(diff.new[1].accepted);

        // Toggle it back
        diff.toggle(1).unwrap();
        assert!(diff.new[0].accepted);

        // Invalid index
        assert!(diff.toggle(0).is_err());
        assert!(diff.toggle(3).is_err());
    }

    #[test]
    fn test_diff_apply_save() {
        let recs = make_workspace_recs(vec![
            ("foo", vec![Condition::Always]),
            ("bar", vec![Condition::Always]),
        ]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        // Add a stale extension
        config.add_extension_with_conditions(
            ComponentSource::Builtin("old".to_string()),
            true,
            vec![],
        );

        let mut diff = RecommendationDiff::compute(&recs, &config);
        // Toggle bar to not accept
        diff.toggle(2).unwrap();

        diff.apply_save(&mut config);

        // foo should be enabled
        assert!(config.is_extension_enabled(&ComponentSource::Builtin("foo".to_string())));
        // bar should be disabled (toggled)
        assert!(!config.is_extension_enabled(&ComponentSource::Builtin("bar".to_string())));
        // old should be removed
        assert!(!config.extensions.contains_key(&ComponentSource::Builtin("old".to_string()).to_config_key()));
    }

    #[test]
    fn test_diff_apply_ignore() {
        let recs = make_workspace_recs(vec![
            ("foo", vec![Condition::Always]),
        ]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        config.add_extension_with_conditions(
            ComponentSource::Builtin("old".to_string()),
            true,
            vec![],
        );

        let diff = RecommendationDiff::compute(&recs, &config);
        diff.apply_ignore(&mut config);

        // foo should be added but disabled
        assert!(!config.is_extension_enabled(&ComponentSource::Builtin("foo".to_string())));
        // But it should be in the config (so we don't ask again)
        assert!(config.extensions.contains_key(&ComponentSource::Builtin("foo".to_string()).to_config_key()));
        // old should be removed
        assert!(!config.extensions.contains_key(&ComponentSource::Builtin("old".to_string()).to_config_key()));
    }

    #[test]
    fn test_condition_explanations() {
        let cond = Condition::FileExists { path: "Cargo.toml".to_string() };
        assert_eq!(cond.explain_why_added(), Some("because a file `Cargo.toml` exists".to_string()));
        assert_eq!(cond.explain_why_stale(), Some("because no file `Cargo.toml` exists".to_string()));

        let cond = Condition::GlobMatches { pattern: "*.rs".to_string() };
        assert_eq!(cond.explain_why_added(), Some("because files matching `*.rs` exist".to_string()));
        assert_eq!(cond.explain_why_stale(), Some("because no files matching `*.rs` exist".to_string()));

        let cond = Condition::Always;
        assert_eq!(cond.explain_why_added(), None);
        assert_eq!(cond.explain_why_stale(), None);
    }
}
