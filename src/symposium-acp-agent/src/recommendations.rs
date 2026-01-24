//! Recommendations - what components to suggest for a workspace
//!
//! This module handles recommending extensions based on workspace
//! characteristics. Recommendations are loaded from a built-in TOML file that
//! is embedded in the binary.

use crate::registry::ComponentSource;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Built-in recommendations TOML, embedded at compile time
const BUILTIN_RECOMMENDATIONS_TOML: &str = include_str!("builtin_recommendations.toml");

/// A recommendation for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// The source of the component (this IS the identity)
    pub source: ComponentSource,

    /// Conditions that must be met for this recommendation to apply
    #[serde(default)]
    pub when: Option<When>,
}

impl Recommendation {
    /// Get the display name for this recommendation
    pub fn display_name(&self) -> String {
        self.source.display_name()
    }

    /// Explain why this recommendation should be added (for new recommendations)
    pub fn explain_why_added(&self) -> Vec<String> {
        self.when
            .as_ref()
            .map(|w| w.explain_why_added())
            .unwrap_or_default()
    }

    /// Explain why this recommendation is stale (for removed recommendations)
    pub fn explain_why_stale(&self) -> Vec<String> {
        self.when
            .as_ref()
            .map(|w| w.explain_why_stale())
            .unwrap_or_default()
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
///
/// Multiple fields at the same level are combined with AND.
/// Use `any` for OR logic, `all` for explicit AND grouping.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct When {
    /// Single file must exist in workspace root
    #[serde(default)]
    pub file_exists: Option<String>,

    /// All files must exist in workspace root (AND)
    #[serde(default)]
    pub files_exist: Option<Vec<String>>,

    /// Single crate must be a dependency
    #[serde(default)]
    pub using_crate: Option<String>,

    /// All crates must be dependencies (AND)
    #[serde(default)]
    pub using_crates: Option<Vec<String>>,

    /// Pattern must match in files
    #[serde(default)]
    pub grep: Option<GrepCondition>,

    /// Any of these conditions must match (OR)
    #[serde(default)]
    pub any: Option<Vec<When>>,

    /// All of these conditions must match (explicit AND)
    #[serde(default)]
    pub all: Option<Vec<When>>,
}

/// Condition for grep pattern matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepCondition {
    /// Pattern to search for
    pub pattern: String,

    /// Path glob to search in (defaults to "*")
    #[serde(default)]
    pub path: Option<String>,
}

impl When {
    /// Check if this condition is met for the given workspace
    pub fn is_met(&self, workspace_path: &Path) -> bool {
        // All specified conditions must be true (AND semantics)
        let mut conditions_checked = false;

        // file-exists
        if let Some(path) = &self.file_exists {
            conditions_checked = true;
            if !workspace_path.join(path).exists() {
                return false;
            }
        }

        // files-exist (all must exist)
        if let Some(paths) = &self.files_exist {
            conditions_checked = true;
            for path in paths {
                if !workspace_path.join(path).exists() {
                    return false;
                }
            }
        }

        // using-crate
        if let Some(_crate_name) = &self.using_crate {
            conditions_checked = true;
            // TODO: Implement Cargo.lock parsing to check if crate is a dependency
            // For now, just check if Cargo.toml exists (basic heuristic)
            if !workspace_path.join("Cargo.toml").exists() {
                return false;
            }
        }

        // using-crates (all must be dependencies)
        if let Some(_crate_names) = &self.using_crates {
            conditions_checked = true;
            // TODO: Implement Cargo.lock parsing to check if all crates are dependencies
            // For now, just check if Cargo.toml exists (basic heuristic)
            if !workspace_path.join("Cargo.toml").exists() {
                return false;
            }
        }

        // grep
        if let Some(_grep) = &self.grep {
            conditions_checked = true;
            // TODO: Implement file content searching
            // For now, always return true (we checked that it's specified)
        }

        // any (OR - at least one must match)
        if let Some(conditions) = &self.any {
            conditions_checked = true;
            if !conditions.iter().any(|c| c.is_met(workspace_path)) {
                return false;
            }
        }

        // all (explicit AND - all must match)
        if let Some(conditions) = &self.all {
            conditions_checked = true;
            if !conditions.iter().all(|c| c.is_met(workspace_path)) {
                return false;
            }
        }

        // If no conditions were specified, always recommend
        if !conditions_checked {
            return true;
        }

        true
    }

    /// Explain why this condition causes a recommendation to be added
    pub fn explain_why_added(&self) -> Vec<String> {
        let mut reasons = Vec::new();

        if let Some(path) = &self.file_exists {
            reasons.push(format!("because `{path}` exists"));
        }

        if let Some(paths) = &self.files_exist {
            for path in paths {
                reasons.push(format!("because `{path}` exists"));
            }
        }

        if let Some(crate_name) = &self.using_crate {
            reasons.push(format!("because using crate `{crate_name}`"));
        }

        if let Some(crate_names) = &self.using_crates {
            for name in crate_names {
                reasons.push(format!("because using crate `{name}`"));
            }
        }

        if let Some(grep) = &self.grep {
            let path = grep.path.as_deref().unwrap_or("*");
            reasons.push(format!("because `{}` found in `{}`", grep.pattern, path));
        }

        if let Some(conditions) = &self.any {
            // For 'any', just list one that matches
            for c in conditions {
                let sub_reasons = c.explain_why_added();
                if !sub_reasons.is_empty() {
                    reasons.extend(sub_reasons);
                    break; // Only need to explain one matching condition
                }
            }
        }

        if let Some(conditions) = &self.all {
            for c in conditions {
                reasons.extend(c.explain_why_added());
            }
        }

        reasons
    }

    /// Explain why this condition causes a recommendation to be stale
    pub fn explain_why_stale(&self) -> Vec<String> {
        let mut reasons = Vec::new();

        if let Some(path) = &self.file_exists {
            reasons.push(format!("because `{path}` no longer exists"));
        }

        if let Some(paths) = &self.files_exist {
            for path in paths {
                reasons.push(format!("because `{path}` no longer exists"));
            }
        }

        if let Some(crate_name) = &self.using_crate {
            reasons.push(format!("because no longer using crate `{crate_name}`"));
        }

        if let Some(crate_names) = &self.using_crates {
            for name in crate_names {
                reasons.push(format!("because no longer using crate `{name}`"));
            }
        }

        if let Some(grep) = &self.grep {
            let path = grep.path.as_deref().unwrap_or("*");
            reasons.push(format!(
                "because `{}` no longer found in `{}`",
                grep.pattern, path
            ));
        }

        if let Some(conditions) = &self.any {
            // For 'any', all must fail for it to be stale
            for c in conditions {
                reasons.extend(c.explain_why_stale());
            }
        }

        if let Some(conditions) = &self.all {
            // For 'all', any one failing makes it stale
            for c in conditions {
                let sub_reasons = c.explain_why_stale();
                if !sub_reasons.is_empty() {
                    reasons.extend(sub_reasons);
                    break;
                }
            }
        }

        reasons
    }
}

/// The recommendations file format
#[derive(Debug, Clone, Deserialize)]
struct RecommendationsFile {
    /// Recommendations list
    #[serde(rename = "recommendation")]
    recommendations: Vec<Recommendation>,
}

/// Loaded recommendations
#[derive(Debug, Clone)]
pub struct Recommendations {
    /// All extension recommendations
    pub extensions: Vec<Recommendation>,
}

impl Recommendations {
    /// Create empty recommendations (for testing)
    pub fn empty() -> Self {
        Self { extensions: vec![] }
    }

    /// Load the built-in recommendations
    pub fn load_builtin() -> Result<Self> {
        Self::from_toml(BUILTIN_RECOMMENDATIONS_TOML)
    }

    /// Parse recommendations from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        let file: RecommendationsFile =
            toml::from_str(toml_str).context("Failed to parse recommendations TOML")?;

        Ok(Self {
            extensions: file.recommendations,
        })
    }

    /// Get recommendations that apply to a specific workspace
    pub fn for_workspace(&self, workspace_path: &Path) -> WorkspaceRecommendations {
        let extensions: Vec<Recommendation> = self
            .extensions
            .iter()
            .filter(|r| r.when.as_ref().map(|w| w.is_met(workspace_path)).unwrap_or(true))
            .cloned()
            .collect();

        WorkspaceRecommendations { extensions }
    }
}

/// Recommendations filtered for a specific workspace
#[derive(Debug, Clone)]
pub struct WorkspaceRecommendations {
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
                format!(
                    "[even though {}]",
                    reasons.join(", ").trim_start_matches("because ")
                )
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
    pub when: Option<When>,
}

impl StaleExtension {
    /// Format this stale extension for display
    pub fn format_display(&self) -> String {
        let name = self.source.display_name();
        let reasons: Vec<String> = self
            .when
            .as_ref()
            .map(|w| w.explain_why_stale())
            .unwrap_or_default();

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
                let when = config.extension_when(&source);
                Some(StaleExtension { source, when })
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
            config.add_extension_with_when(
                rec.recommendation.source.clone(),
                rec.accepted, // enabled = accepted
                rec.recommendation.when.clone(),
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
            config.add_extension_with_when(
                rec.recommendation.source.clone(),
                false, // Always disabled for IGNORE
                rec.recommendation.when.clone(),
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

        // Should have some extension recommendations
        assert!(
            !recs.extensions.is_empty(),
            "Should have extension recommendations"
        );

        // Should have sparkle (always recommended)
        assert!(
            recs.extensions
                .iter()
                .any(|r| r.source == ComponentSource::Builtin("sparkle".to_string())),
            "Should have sparkle recommendation"
        );
    }

    #[test]
    fn test_workspace_filtering() {
        let toml = r#"
[[recommendation]]
source.builtin = "always-on"

[[recommendation]]
source.builtin = "rust-only"
when.file-exists = "Cargo.toml"
"#;

        let recs = Recommendations::from_toml(toml).unwrap();

        // Create a temp directory without Cargo.toml
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());

        // Should only have the "always-on" extension
        assert_eq!(workspace_recs.extensions.len(), 1);
        assert_eq!(workspace_recs.extensions[0].display_name(), "always-on");

        // Now create Cargo.toml
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());

        // Should have both extensions
        assert_eq!(workspace_recs.extensions.len(), 2);
    }

    #[test]
    fn test_when_any_condition() {
        let toml = r#"
[[recommendation]]
source.builtin = "multi-lang"
when.any = [
    { file-exists = "Cargo.toml" },
    { file-exists = "package.json" },
]
"#;

        let recs = Recommendations::from_toml(toml).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();

        // No matching files
        let workspace_recs = recs.for_workspace(temp_dir.path());
        assert_eq!(workspace_recs.extensions.len(), 0);

        // Create Cargo.toml
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());
        assert_eq!(workspace_recs.extensions.len(), 1);

        // Remove Cargo.toml, create package.json
        std::fs::remove_file(temp_dir.path().join("Cargo.toml")).unwrap();
        std::fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());
        assert_eq!(workspace_recs.extensions.len(), 1);
    }

    #[test]
    fn test_when_multiple_conditions_and() {
        let toml = r#"
[[recommendation]]
source.builtin = "both-required"
when.file-exists = "Cargo.toml"
when.files-exist = ["src/lib.rs"]
"#;

        let recs = Recommendations::from_toml(toml).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();

        // Neither file
        let workspace_recs = recs.for_workspace(temp_dir.path());
        assert_eq!(workspace_recs.extensions.len(), 0);

        // Only Cargo.toml
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());
        assert_eq!(workspace_recs.extensions.len(), 0);

        // Both files
        std::fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        std::fs::write(temp_dir.path().join("src/lib.rs"), "").unwrap();
        let workspace_recs = recs.for_workspace(temp_dir.path());
        assert_eq!(workspace_recs.extensions.len(), 1);
    }

    // ========================================================================
    // Diff tests
    // ========================================================================

    fn make_workspace_recs(extensions: Vec<(&str, Option<When>)>) -> WorkspaceRecommendations {
        WorkspaceRecommendations {
            extensions: extensions
                .into_iter()
                .map(|(name, when)| Recommendation {
                    source: ComponentSource::Builtin(name.to_string()),
                    when,
                })
                .collect(),
        }
    }

    #[test]
    fn test_diff_new_recommendations() {
        let recs = make_workspace_recs(vec![
            (
                "foo",
                Some(When {
                    file_exists: Some("Cargo.toml".to_string()),
                    ..Default::default()
                }),
            ),
            ("bar", None),
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
        config.add_extension_with_when(
            ComponentSource::Builtin("old-ext".to_string()),
            true,
            Some(When {
                file_exists: Some("old.txt".to_string()),
                ..Default::default()
            }),
        );

        let diff = RecommendationDiff::compute(&recs, &config);

        assert!(diff.new.is_empty());
        assert_eq!(diff.stale.len(), 1);
        assert_eq!(
            diff.stale[0].source,
            ComponentSource::Builtin("old-ext".to_string())
        );
        assert!(diff.has_changes());
        assert!(diff.is_stale_only());
    }

    #[test]
    fn test_diff_no_changes_when_in_sync() {
        let recs = make_workspace_recs(vec![("foo", None)]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        // Add the same extension that's recommended
        config.add_extension_with_when(ComponentSource::Builtin("foo".to_string()), true, None);

        let diff = RecommendationDiff::compute(&recs, &config);

        assert!(diff.new.is_empty());
        assert!(diff.stale.is_empty());
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_diff_disabled_extension_not_new() {
        // If an extension is in config but disabled, it's still "known" - not new
        let recs = make_workspace_recs(vec![("foo", None)]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        config.add_extension_with_when(
            ComponentSource::Builtin("foo".to_string()),
            false, // Disabled
            None,
        );

        let diff = RecommendationDiff::compute(&recs, &config);

        // foo is not new because it's already in config (even though disabled)
        assert!(diff.new.is_empty());
        assert!(diff.stale.is_empty());
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_diff_toggle() {
        let recs = make_workspace_recs(vec![("foo", None), ("bar", None)]);
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
        let recs = make_workspace_recs(vec![("foo", None), ("bar", None)]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        // Add a stale extension
        config.add_extension_with_when(
            ComponentSource::Builtin("old".to_string()),
            true,
            None,
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
        assert!(!config
            .extensions
            .contains_key(&ComponentSource::Builtin("old".to_string()).to_config_key()));
    }

    #[test]
    fn test_diff_apply_ignore() {
        let recs = make_workspace_recs(vec![("foo", None)]);
        let mut config = WorkspaceConfig::new(
            ComponentSource::Builtin("agent".to_string()),
            vec![],
        );
        config.add_extension_with_when(
            ComponentSource::Builtin("old".to_string()),
            true,
            None,
        );

        let diff = RecommendationDiff::compute(&recs, &config);
        diff.apply_ignore(&mut config);

        // foo should be added but disabled
        assert!(!config.is_extension_enabled(&ComponentSource::Builtin("foo".to_string())));
        // But it should be in the config (so we don't ask again)
        assert!(config
            .extensions
            .contains_key(&ComponentSource::Builtin("foo".to_string()).to_config_key()));
        // old should be removed
        assert!(!config
            .extensions
            .contains_key(&ComponentSource::Builtin("old".to_string()).to_config_key()));
    }

    #[test]
    fn test_when_explanations() {
        let when = When {
            file_exists: Some("Cargo.toml".to_string()),
            ..Default::default()
        };
        let added = when.explain_why_added();
        assert_eq!(added, vec!["because `Cargo.toml` exists"]);

        let stale = when.explain_why_stale();
        assert_eq!(stale, vec!["because `Cargo.toml` no longer exists"]);
    }
}
