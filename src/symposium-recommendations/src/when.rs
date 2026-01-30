//! Condition types for when recommendations apply

use serde::{Deserialize, Serialize};

/// Conditions for when a recommendation applies
///
/// Multiple fields at the same level are combined with AND.
/// Use `any` for OR logic, `all` for explicit AND grouping.
///
/// # Examples
///
/// Single file check:
/// ```toml
/// when.file-exists = "Cargo.toml"
/// ```
///
/// Multiple conditions (AND):
/// ```toml
/// when.file-exists = "Cargo.toml"
/// when.using-crate = "serde"
/// ```
///
/// OR conditions:
/// ```toml
/// when.any = [
///     { file-exists = "Cargo.toml" },
///     { file-exists = "package.json" },
/// ]
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct When {
    /// Single file must exist in workspace root
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_exists: Option<String>,

    /// All files must exist in workspace root (AND)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_exist: Option<Vec<String>>,

    /// Single crate must be a dependency
    #[serde(skip_serializing_if = "Option::is_none")]
    pub using_crate: Option<String>,

    /// All crates must be dependencies (AND)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub using_crates: Option<Vec<String>>,

    /// Any of these conditions must match (OR)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any: Option<Vec<When>>,

    /// All of these conditions must match (explicit AND)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<Vec<When>>,
}

impl When {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_when_serialization() {
        let when = When {
            file_exists: Some("Cargo.toml".to_string()),
            ..Default::default()
        };

        let toml = toml::to_string(&when).unwrap();
        assert!(toml.contains("file-exists"));

        let parsed: When = toml::from_str(&toml).unwrap();
        assert_eq!(when, parsed);
    }

    #[test]
    fn test_explain_why_added() {
        let when = When {
            file_exists: Some("Cargo.toml".to_string()),
            using_crate: Some("serde".to_string()),
            ..Default::default()
        };

        let reasons = when.explain_why_added();
        assert_eq!(reasons.len(), 2);
        assert!(reasons[0].contains("Cargo.toml"));
        assert!(reasons[1].contains("serde"));
    }

    #[test]
    fn test_any_condition() {
        let when = When {
            any: Some(vec![
                When {
                    file_exists: Some("Cargo.toml".to_string()),
                    ..Default::default()
                },
                When {
                    file_exists: Some("package.json".to_string()),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        // Serializes correctly
        let toml = toml::to_string(&when).unwrap();
        let parsed: When = toml::from_str(&toml).unwrap();
        assert_eq!(when, parsed);
    }
}
