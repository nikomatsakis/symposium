use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::git_source::UpdateLevel;
use crate::hook::HookEvent;

/// Source declaration for remote plugin artifacts.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PluginSource {
    /// Path on the local filesystem.
    pub path: Option<PathBuf>,

    /// GitHub URL pointing to a directory in a repository.
    pub git: Option<String>,
}

/// A `[[skills]]` entry from a plugin manifest.
///
/// Each group declares which crates it advises on (`advice-for`), workspace
/// constraints (`applies-when`), an activation mode, and optionally a remote
/// source for the skill files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillGroup {
    /// Crate atoms this group advises on (e.g., `["serde", "serde_json>=1.0"]`).
    #[serde(default, rename = "advice-for")]
    pub advice_for: Option<Vec<String>>,
    /// Workspace constraints: all listed crate atoms must be present (AND semantics).
    #[serde(default, rename = "applies-when")]
    pub applies_when: Option<Vec<String>>,
    /// Activation mode for skills in this group ("default" or "optional").
    pub activation: Option<String>,
    /// Remote source for skills.
    #[serde(default)]
    pub source: PluginSource,
}

/// A parsed plugin with its path and manifest.
#[derive(Debug, Clone)]
pub struct ParsedPlugin {
    /// The path from which the plugin was parsed.
    pub path: PathBuf,

    /// The parsed plugin manifest.
    pub plugin: Plugin,
}

/// A loaded plugin manifest with hooks and skill groups.
///
/// This is a table of contents — it describes what skills and hooks are
/// available, but does not load skill content. The skills layer handles
/// discovery and loading.
#[derive(Debug, Clone, Serialize)]
pub struct Plugin {
    pub name: String,
    pub installation: Option<Installation>,
    pub hooks: Vec<Hook>,
    pub skills: Vec<SkillGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Installation {
    pub commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hook {
    pub name: String,
    pub event: HookEvent,
    pub matcher: Option<String>,
    pub command: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub source_type: &'static str,
    pub git_url: Option<String>,
    pub path: Option<String>,
    pub plugins: Vec<PluginInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub hooks_count: usize,
    pub skill_groups_count: usize,
}

/// Raw TOML manifest deserialized from a plugin `.toml` file.
#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: String,
    #[serde(default)]
    installation: Option<Installation>,
    #[serde(default)]
    hooks: Vec<Hook>,
    #[serde(default)]
    skills: Vec<SkillGroup>,
}

/// Fetch/update git-based plugin sources.
///
/// Ensure git-based plugin sources are up to date.
///
/// `update` controls freshness checking behavior (see `UpdateLevel`).
/// Only refreshes sources with `auto-update = true` (unless `update` is `Fetch`).
/// Path-based sources are skipped (no fetching needed).
pub async fn ensure_plugin_sources(update: UpdateLevel) {
    let sources = crate::config::plugin_sources();

    for source in &sources {
        if !matches!(update, UpdateLevel::Fetch) && !source.auto_update {
            tracing::debug!(source = %source.name, "skipping (auto-update disabled)");
            continue;
        }

        let Some(ref git_url) = source.git else {
            tracing::debug!(source = %source.name, "skipping (can only auto-update git)");
            continue;
        };

        tracing::debug!(source = %source.name, url = %git_url, "ensuring plugin source");

        match fetch_plugin_source(git_url, update).await {
            Ok(path) => {
                tracing::debug!(source = %source.name, path = %path.display(), "plugin source ready");
            }
            Err(e) => {
                tracing::warn!(source = %source.name, git_url = %git_url, error = %e, "failed to fetch plugin source");
            }
        }
    }
}

/// Load all plugins from all configured plugin source directories,
/// discarding load errors with warnings.
pub fn load_all_plugins() -> Vec<ParsedPlugin> {
    let plugin_results = match load_all_plugin_results() {
        Ok(ps) => ps,

        Err(e) => {
            tracing::warn!(error = %e, "failed to load plugins");
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for plugin_res in plugin_results {
        match plugin_res {
            Ok(p) => out.push(p),
            Err(e) => {
                tracing::warn!(error = %e, "failed to load plugin");
            }
        }
    }
    out
}

/// Sync plugin sources.
///
/// If `provider` is Some, sync only that provider (ignores auto-update).
/// If `provider` is None, sync all sources with auto-update = true.
pub async fn sync_plugin_source(provider: Option<&str>) -> Result<Vec<String>> {
    let sources = crate::config::plugin_sources();
    let mut synced = Vec::new();

    for source in &sources {
        if let Some(name) = provider {
            if source.name != name {
                continue;
            }
        } else if !source.auto_update {
            tracing::debug!(source = %source.name, "skipping (auto-update disabled)");
            continue;
        }

        if let Some(ref git_url) = source.git {
            tracing::debug!(source = %source.name, url = %git_url, "syncing plugin source");
            match fetch_plugin_source(git_url, UpdateLevel::Fetch).await {
                Ok(path) => {
                    tracing::info!(source = %source.name, path = %path.display(), "synced");
                    synced.push(source.name.clone());
                }
                Err(e) => {
                    tracing::warn!(source = %source.name, error = %e, "failed to sync");
                }
            }
        } else {
            tracing::debug!(source = %source.name, "skipping path-based source");
        }
    }

    Ok(synced)
}

/// List all providers and their plugins.
pub fn list_plugins() -> Vec<ProviderInfo> {
    let sources = crate::config::plugin_sources();
    let mut providers = Vec::new();

    for source in &sources {
        let source_path = resolve_plugin_source_dir(source);
        let plugins: Vec<PluginInfo> = source_path
            .and_then(|p| load_plugins_from_dir(&p).ok())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|ParsedPlugin { path: _, plugin: p }| PluginInfo {
                name: p.name,
                hooks_count: p.hooks.len(),
                skill_groups_count: p.skills.len(),
            })
            .collect();

        providers.push(ProviderInfo {
            name: source.name.clone(),
            source_type: if source.git.is_some() { "git" } else { "path" },
            git_url: source.git.clone(),
            path: source.path.clone(),
            plugins,
        });
    }

    providers
}

/// Find a plugin by name across all sources.
pub fn find_plugin(name: &str) -> Option<ParsedPlugin> {
    let sources = crate::config::plugin_sources();

    for source in &sources {
        let source_path = resolve_plugin_source_dir(source);
        if let Some(ref path) = source_path {
            if let Ok(results) = load_plugins_from_dir(path) {
                for result in results {
                    if let Ok(parsed_plugin) = result {
                        if parsed_plugin.plugin.name == name {
                            return Some(parsed_plugin);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Resolve the directories for all configured plugin sources.
///
/// For `path` sources: resolves relative to `config_dir()`, or uses absolute paths as-is.
/// For `git` sources: computes the cache path under `~/.symposium/cache/plugin-sources/`.
///
/// Does no network I/O — just computes paths.
fn resolve_plugin_source_dirs() -> Vec<PathBuf> {
    let sources = crate::config::plugin_sources();
    let config_dir = crate::config::config_dir();
    let cache_base = crate::config::cache_dir().join("plugin-sources");

    let mut dirs = Vec::new();
    for source in &sources {
        if let Some(path) = &source.path {
            let p = PathBuf::from(path);
            if p.is_absolute() {
                dirs.push(p);
            } else {
                dirs.push(config_dir.join(p));
            }
        } else if let Some(git_url) = &source.git {
            match crate::git_source::parse_github_url(git_url) {
                Ok(gh) => dirs.push(cache_base.join(gh.cache_key())),
                Err(e) => {
                    tracing::warn!(source = %source.name, error = %e, "bad plugin source URL");
                }
            }
        } else {
            tracing::warn!(source = %source.name, "plugin source has neither git nor path");
        }
    }
    dirs
}

fn resolve_plugin_source_dir(source: &crate::config::PluginSourceConfig) -> Option<PathBuf> {
    let config_dir = crate::config::config_dir();
    let cache_base = crate::config::cache_dir().join("plugin-sources");

    if let Some(ref path) = source.path {
        let p = PathBuf::from(path);
        if p.is_absolute() {
            return Some(p);
        } else {
            return Some(config_dir.join(p));
        }
    } else if let Some(ref git_url) = source.git {
        match crate::git_source::parse_github_url(git_url) {
            Ok(gh) => return Some(cache_base.join(gh.cache_key())),
            Err(e) => {
                tracing::warn!(source = %source.name, error = %e, "bad plugin source URL");
            }
        }
    }
    None
}

/// Fetch a plugin source repository, returning the cached directory path.
async fn fetch_plugin_source(git_url: &str, update: UpdateLevel) -> Result<PathBuf> {
    use crate::git_source;

    let source = git_source::parse_github_url(git_url)?;
    let cache_mgr = git_source::PluginCacheManager::new("plugin-sources");
    cache_mgr.get_or_fetch(&source, git_url, update).await
}

/// Load all plugins from all configured plugin source directories.
///
/// Scans each directory returned by `resolve_plugin_source_dirs()`.
fn load_all_plugin_results() -> Result<Vec<Result<ParsedPlugin>>> {
    let mut all = Vec::new();
    for dir in resolve_plugin_source_dirs() {
        match load_plugins_from_dir(&dir) {
            Ok(results) => all.extend(results),
            Err(e) => {
                tracing::warn!(dir = %dir.display(), error = %e, "failed to load plugin source dir");
            }
        }
    }
    Ok(all)
}

/// Load all plugins from a directory.
///
/// Plugins are `.toml` files. They can live as standalone files or inside
/// directories (as `symposium.toml`). Either way, the TOML is the plugin.
fn load_plugins_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Result<ParsedPlugin>>> {
    let mut plugins = Vec::new();
    let dir = dir.as_ref();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(plugins),
        Err(e) => return Err(e.into()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml") {
            let plugin = load_plugin(&path)
                .with_context(|| format!("loading plugin from `{}`", path.display()));

            tracing::debug!(
                path = %path.display(),
                plugin = ?plugin,
                "loaded plugin entry",
            );

            plugins.push(plugin);
        } else {
            tracing::debug!(
                path = %path.display(),
                "skipping non-plugin entry in plugins directory"
            );
        }
    }

    Ok(plugins)
}

/// Load a single plugin from a TOML manifest.
///
/// `local_dir` is the containing directory when the manifest lives inside a
/// plugin directory (used as fallback skill directory when no `source.git`).
pub fn load_plugin(manifest_path: &Path) -> Result<ParsedPlugin> {
    let content = fs::read_to_string(manifest_path)?;
    let manifest: PluginManifest = toml::from_str(&content)?;

    Ok(ParsedPlugin {
        path: manifest_path.to_path_buf(),
        plugin: Plugin {
            name: manifest.name,
            installation: manifest.installation,
            hooks: manifest.hooks,
            skills: manifest.skills,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    fn from_str(s: &str) -> Result<Plugin> {
        let manifest: PluginManifest = toml::from_str(s)?;
        Ok(Plugin {
            name: manifest.name,
            installation: manifest.installation,
            hooks: manifest.hooks,
            skills: manifest.skills,
        })
    }

    const SAMPLE: &str = indoc! {r#"
        name = "example-plugin"

        [installation]
        summary = "Download and install helper"
        commands = ["wget https://example.org/bin/tool"]

        [[hooks]]
        name = "test"
        event = "PreToolUse"
        command = "echo open"
    "#};

    #[test]
    fn parse_sample() {
        let plugin = from_str(SAMPLE).expect("parse");
        assert_eq!(plugin.name, "example-plugin");
        assert_eq!(plugin.hooks.len(), 1);
        assert!(plugin.skills.is_empty());
    }

    #[test]
    fn parse_manifest_with_source_git_under_skills() {
        let toml = indoc! {r#"
            name = "remote-plugin"

            [[skills]]
            advice-for = ["serde"]
            applies-when = ["serde>=1.0"]
            source.git = "https://github.com/org/repo/tree/main/serde"
        "#};
        let plugin = from_str(toml).expect("parse");
        assert_eq!(plugin.name, "remote-plugin");
        assert_eq!(plugin.skills.len(), 1);
        let group = &plugin.skills[0];
        assert_eq!(
            group.advice_for.as_deref(),
            Some(["serde".to_string()].as_slice())
        );
        assert_eq!(
            group.applies_when.as_deref(),
            Some(["serde>=1.0".to_string()].as_slice())
        );
        assert_eq!(
            group.source.git.as_ref().map(|s| s.as_str()),
            Some("https://github.com/org/repo/tree/main/serde")
        );
    }

    #[test]
    fn parse_manifest_with_multiple_skill_groups() {
        let toml = indoc! {r#"
            name = "multi-group"

            [[skills]]
            advice-for = ["serde"]
            applies-when = ["serde>=1.0"]

            [[skills]]
            advice-for = ["tokio"]
            applies-when = ["tokio>=1.0"]
        "#};
        let plugin = from_str(toml).expect("parse");
        assert_eq!(plugin.name, "multi-group");
        assert_eq!(plugin.skills.len(), 2);
        assert_eq!(
            plugin.skills[0].advice_for.as_deref(),
            Some(["serde".to_string()].as_slice())
        );
        assert_eq!(
            plugin.skills[1].advice_for.as_deref(),
            Some(["tokio".to_string()].as_slice())
        );
    }
}
