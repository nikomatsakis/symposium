//! User configuration types for Symposium.
//!
//! Configuration is stored per-workspace at:
//! `~/.symposium/config/<encoded-workspace-path>/config.json`
//!
//! The configuration uses `ComponentSource` as the identity for both
//! agents and extensions, enabling easy diffing with recommendations.

use crate::registry::ComponentSource;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Extension configuration entry
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExtensionConfig {
    /// Whether this extension is enabled
    pub enabled: bool,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Per-workspace configuration for Symposium.
///
/// Uses `ComponentSource` as identity for both agent and extensions.
/// This makes it easy to compare with recommendations and detect changes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    /// The agent to use for this workspace
    pub agent: ComponentSource,

    /// Extensions with their enabled state
    /// The key is the JSON-serialized ComponentSource
    #[serde(default)]
    pub extensions: BTreeMap<String, ExtensionConfig>,
}

// ============================================================================
// Global Agent Config (for default agent across workspaces)
// ============================================================================

/// Global agent configuration.
///
/// Stores the user's default agent choice. This is used to populate the initial
/// agent for new workspaces. Each workspace can override this independently.
///
/// Stored at `~/.symposium/config/agent.json`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct GlobalAgentConfig {
    /// The default agent to use for new workspaces
    pub agent: ComponentSource,
}

impl GlobalAgentConfig {
    /// Create a new global agent config
    pub fn new(agent: ComponentSource) -> Self {
        Self { agent }
    }

    /// Get the path to the global agent config file
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".symposium").join("config").join("agent.json"))
    }

    /// Load the global agent config. Returns None if it doesn't exist.
    pub fn load() -> Result<Option<Self>> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read global agent config from {}", path.display()))?;
        let config: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse global agent config from {}", path.display()))?;
        Ok(Some(config))
    }

    /// Save the global agent config
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create config directory {}", dir.display()))?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write global agent config to {}", path.display()))?;
        Ok(())
    }
}

// ============================================================================
// Workspace Config
// ============================================================================

impl WorkspaceConfig {
    /// Create a new workspace config with the given agent and extensions
    pub fn new(agent: ComponentSource, extensions: Vec<ComponentSource>) -> Self {
        let extensions = extensions
            .into_iter()
            .map(|source| {
                let key = source.to_config_key();
                (key, ExtensionConfig::default())
            })
            .collect();

        Self { agent, extensions }
    }

    /// Get the config directory for a workspace
    pub fn workspace_config_dir(workspace_path: &Path) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let encoded = encode_path(workspace_path);
        Ok(home.join(".symposium").join("config").join(encoded))
    }

    /// Get the config file path for a workspace
    pub fn config_path(workspace_path: &Path) -> Result<PathBuf> {
        Ok(Self::workspace_config_dir(workspace_path)?.join("config.json"))
    }

    /// Load config for a workspace. Returns None if config doesn't exist.
    pub fn load(workspace_path: &Path) -> Result<Option<Self>> {
        let path = Self::config_path(workspace_path)?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;
        Ok(Some(config))
    }

    /// Save config for a workspace
    pub fn save(&self, workspace_path: &Path) -> Result<()> {
        let path = Self::config_path(workspace_path)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create config directory {}", dir.display()))?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    /// Get enabled extension sources in order
    pub fn enabled_extensions(&self) -> Vec<ComponentSource> {
        self.extensions
            .iter()
            .filter(|(_, config)| config.enabled)
            .filter_map(|(key, _)| ComponentSource::from_config_key(key).ok())
            .collect()
    }

    /// Add an extension (enabled by default)
    pub fn add_extension(&mut self, source: ComponentSource) {
        let key = source.to_config_key();
        self.extensions
            .entry(key)
            .or_insert(ExtensionConfig::default());
    }

    /// Remove an extension
    pub fn remove_extension(&mut self, source: &ComponentSource) {
        let key = source.to_config_key();
        self.extensions.remove(&key);
    }

    /// Toggle an extension's enabled state
    pub fn toggle_extension(&mut self, source: &ComponentSource) -> bool {
        let key = source.to_config_key();
        if let Some(config) = self.extensions.get_mut(&key) {
            config.enabled = !config.enabled;
            config.enabled
        } else {
            // Add it if it doesn't exist
            self.extensions
                .insert(key, ExtensionConfig { enabled: true });
            true
        }
    }

    /// Set an extension's enabled state
    pub fn set_extension_enabled(&mut self, source: &ComponentSource, enabled: bool) {
        let key = source.to_config_key();
        if let Some(config) = self.extensions.get_mut(&key) {
            config.enabled = enabled;
        } else {
            self.extensions.insert(key, ExtensionConfig { enabled });
        }
    }

    /// Check if an extension is enabled
    pub fn is_extension_enabled(&self, source: &ComponentSource) -> bool {
        let key = source.to_config_key();
        self.extensions
            .get(&key)
            .map(|c| c.enabled)
            .unwrap_or(false)
    }
}

/// Encode a path for use as a directory name.
/// Uses base64 with URL-safe characters.
fn encode_path(path: &Path) -> String {
    use base64::Engine;
    let path_str = path.to_string_lossy();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(path_str.as_bytes())
}

/// Decode a directory name back to a path.
#[allow(dead_code)]
fn decode_path(encoded: &str) -> Result<PathBuf> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .context("Failed to decode path")?;
    let path_str = String::from_utf8(bytes).context("Path is not valid UTF-8")?;
    Ok(PathBuf::from(path_str))
}

// ============================================================================
// Legacy types for backwards compatibility
// ============================================================================

/// Legacy user configuration for Symposium.
/// Used for migration from old config format.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SymposiumUserConfig {
    /// Downstream agent command (shell words, e.g., "npx -y @anthropic-ai/claude-code-acp")
    pub agent: String,

    /// Proxy extensions to enable
    pub proxies: Vec<ProxyEntry>,
}

/// A proxy extension entry in the legacy configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ProxyEntry {
    /// Proxy name (e.g., "sparkle", "ferris", "cargo")
    pub name: String,

    /// Whether this proxy is enabled
    pub enabled: bool,
}

impl SymposiumUserConfig {
    /// Get the legacy config directory path: ~/.symposium/
    pub fn dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".symposium"))
    }

    /// Get the legacy config file path: ~/.symposium/config.jsonc
    pub fn path() -> Result<PathBuf> {
        Ok(Self::dir()?.join("config.jsonc"))
    }

    /// Load legacy config from the given path, or the default path if None.
    /// Returns None if the config file doesn't exist.
    pub fn load(path: Option<impl AsRef<std::path::Path>>) -> Result<Option<Self>> {
        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => Self::path()?,
        };
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Self = serde_jsonc::from_str(&content)?;
        Ok(Some(config))
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::path()?)
    }

    /// Save config to a specific path.
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the list of enabled proxy names.
    pub fn enabled_proxies(&self) -> Vec<String> {
        self.proxies
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.name.clone())
            .collect()
    }

    /// Parse the agent string into command arguments (shell words).
    pub fn agent_args(&self) -> Result<Vec<String>> {
        shell_words::split(&self.agent)
            .map_err(|e| anyhow::anyhow!("Failed to parse agent command: {}", e))
    }

    /// Create a default config with all proxies enabled.
    pub fn with_agent(agent: impl Into<String>) -> Self {
        Self {
            agent: agent.into(),
            proxies: vec![
                ProxyEntry {
                    name: "sparkle".to_string(),
                    enabled: true,
                },
                ProxyEntry {
                    name: "ferris".to_string(),
                    enabled: true,
                },
                ProxyEntry {
                    name: "cargo".to_string(),
                    enabled: true,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{CargoDistribution, NpxDistribution};
    use std::collections::BTreeMap;

    #[test]
    fn test_workspace_config_new() {
        let agent = ComponentSource::Npx(NpxDistribution {
            package: "@zed-industries/claude-code-acp@latest".to_string(),
            args: vec![],
            env: BTreeMap::new(),
        });
        let extensions = vec![
            ComponentSource::Builtin("ferris".to_string()),
            ComponentSource::Cargo(CargoDistribution {
                crate_name: "sparkle-mcp".to_string(),
                version: None,
                binary: None,
                args: vec!["--acp".to_string()],
            }),
        ];

        let config = WorkspaceConfig::new(agent.clone(), extensions);

        assert_eq!(config.agent, agent);
        assert_eq!(config.extensions.len(), 2);
        assert!(config.is_extension_enabled(&ComponentSource::Builtin("ferris".to_string())));
    }

    #[test]
    fn test_workspace_config_save_load_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path();

        let agent = ComponentSource::Builtin("eliza".to_string());
        let extensions = vec![ComponentSource::Builtin("ferris".to_string())];
        let config = WorkspaceConfig::new(agent.clone(), extensions);

        // Save
        config.save(workspace_path).unwrap();

        // Load
        let loaded = WorkspaceConfig::load(workspace_path).unwrap().unwrap();

        assert_eq!(config, loaded);
    }

    #[test]
    fn test_workspace_config_toggle_extension() {
        let agent = ComponentSource::Builtin("eliza".to_string());
        let mut config = WorkspaceConfig::new(agent, vec![]);

        let ext = ComponentSource::Builtin("ferris".to_string());

        // Initially not present
        assert!(!config.is_extension_enabled(&ext));

        // Toggle on
        assert!(config.toggle_extension(&ext));
        assert!(config.is_extension_enabled(&ext));

        // Toggle off
        assert!(!config.toggle_extension(&ext));
        assert!(!config.is_extension_enabled(&ext));
    }

    #[test]
    fn test_encode_decode_path() {
        let original = PathBuf::from("/Users/test/my-project");
        let encoded = encode_path(&original);
        let decoded = decode_path(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_enabled_extensions_order() {
        let agent = ComponentSource::Builtin("eliza".to_string());
        let extensions = vec![
            ComponentSource::Builtin("a".to_string()),
            ComponentSource::Builtin("b".to_string()),
            ComponentSource::Builtin("c".to_string()),
        ];
        let mut config = WorkspaceConfig::new(agent, extensions);

        // Disable b
        config.set_extension_enabled(&ComponentSource::Builtin("b".to_string()), false);

        let enabled = config.enabled_extensions();
        assert_eq!(enabled.len(), 2);
        // BTreeMap orders by key, so order is deterministic
    }
}
