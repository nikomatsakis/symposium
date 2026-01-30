//! Symposium Recommendations
//!
//! Types and parsing for Symposium mod recommendations. This crate provides:
//!
//! - [`Recommendation`] - A recommended mod with source and conditions
//! - [`ComponentSource`] - How to obtain and run a component (cargo, npx, etc.)
//! - [`When`] - Conditions for when a recommendation applies
//!
//! # Example
//!
//! ```
//! use symposium_recommendations::{Recommendations, ComponentSource};
//!
//! let toml = r#"
//! [[recommendation]]
//! source.cargo = { crate = "sparkle-mcp", args = ["--acp"] }
//!
//! [[recommendation]]
//! source.cargo = { crate = "symposium-cargo" }
//! when.file-exists = "Cargo.toml"
//! "#;
//!
//! let recs = Recommendations::from_toml(toml).unwrap();
//! assert_eq!(recs.mods.len(), 2);
//! ```

mod source;
mod when;

pub use source::*;
pub use when::When;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
}

/// The recommendations file format (for parsing TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecommendationsFile {
    #[serde(rename = "recommendation")]
    recommendations: Vec<Recommendation>,
}

/// A collection of recommendations
#[derive(Debug, Clone)]
pub struct Recommendations {
    /// All mod recommendations
    pub mods: Vec<Recommendation>,
}

impl Recommendations {
    /// Create empty recommendations
    pub fn empty() -> Self {
        Self { mods: vec![] }
    }

    /// Parse recommendations from a TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        let file: RecommendationsFile =
            toml::from_str(toml_str).context("Failed to parse recommendations TOML")?;

        Ok(Self {
            mods: file.recommendations,
        })
    }

    /// Parse a single recommendation from a TOML string
    ///
    /// Expected format:
    /// ```toml
    /// [recommendation]
    /// source.cargo = { crate = "example" }
    /// when.file-exists = "Cargo.toml"
    /// ```
    pub fn parse_single(toml_str: &str) -> Result<Recommendation> {
        #[derive(Deserialize)]
        struct SingleFile {
            recommendation: Recommendation,
        }

        let file: SingleFile =
            toml::from_str(toml_str).context("Failed to parse recommendation TOML")?;

        Ok(file.recommendation)
    }

    /// Concatenate multiple TOML recommendation files into one
    ///
    /// Each input should be a single `[recommendation]` block.
    /// Output is a valid recommendations TOML with `[[recommendation]]` array.
    pub fn concatenate_files(files: &[&str]) -> Result<String> {
        let mut all_recs = Vec::new();

        for content in files {
            let rec = Self::parse_single(content)?;
            all_recs.push(rec);
        }

        // Serialize back to TOML
        let output = Recommendations { mods: all_recs };
        let file = RecommendationsFile {
            recommendations: output.mods,
        };

        // toml crate serializes Vec with [[array]] syntax
        toml::to_string_pretty(&file).context("Failed to serialize recommendations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recommendations() {
        let toml = r#"
[[recommendation]]
source.cargo = { crate = "sparkle-mcp", args = ["--acp"] }

[[recommendation]]
source.cargo = { crate = "symposium-cargo" }
when.file-exists = "Cargo.toml"
"#;

        let recs = Recommendations::from_toml(toml).unwrap();
        assert_eq!(recs.mods.len(), 2);
        assert_eq!(recs.mods[0].display_name(), "sparkle-mcp");
        assert!(recs.mods[0].when.is_none());
        assert!(recs.mods[1].when.is_some());
    }

    #[test]
    fn test_parse_single_recommendation() {
        let toml = r#"
[recommendation]
source.cargo = { crate = "example-mod" }
when.file-exists = "package.json"
"#;

        let rec = Recommendations::parse_single(toml).unwrap();
        assert_eq!(rec.display_name(), "example-mod");
        assert_eq!(
            rec.when.as_ref().unwrap().file_exists,
            Some("package.json".to_string())
        );
    }

    #[test]
    fn test_concatenate_files() {
        let file1 = r#"
[recommendation]
source.cargo = { crate = "mod-a" }
"#;

        let file2 = r#"
[recommendation]
source.cargo = { crate = "mod-b" }
when.file-exists = "Cargo.toml"
"#;

        let combined = Recommendations::concatenate_files(&[file1, file2]).unwrap();
        let recs = Recommendations::from_toml(&combined).unwrap();
        assert_eq!(recs.mods.len(), 2);
    }
}
