use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::config::{home_dir, plugins_dir};
use crate::hook::HookEvent;

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub name: String,
    #[serde(default)]
    pub installation: Option<Installation>,
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    pub commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Hook {
    pub name: String,
    pub event: HookEvent,
    pub command: String,
}

/// Load all plugins from a directory containing TOML plugin files.
pub fn load_plugins_from_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<Result<Plugin>>> {
    let mut plugins = Vec::new();
    let dir = dir.as_ref();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            plugins.push(Err(anyhow::anyhow!(
                "directory contains non-file entry: {}",
                path.display()
            )));
        }

        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => {
                plugins.push(from_path(&path));
            }
            other => {
                plugins.push(Err(anyhow::anyhow!(
                    "unexpected file extension for {}: {:?}",
                    path.display(),
                    other
                )));
            }
        }
    }
    Ok(plugins)
}

pub fn load_global_plugins() -> Result<Vec<Result<Plugin>>> {
    let dir = plugins_dir();
    load_plugins_from_dir(dir)
}

pub fn from_str(s: &str) -> Result<Plugin> {
    let p: Plugin = toml::from_str(s)?;
    Ok(p)
}

pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Plugin> {
    let s = fs::read_to_string(path)?;
    from_str(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
name = "example-plugin"

[installation]
summary = "Download and install helper"
commands = ["wget https://example.org/bin/tool"]

[[hooks]]
name = "test"
event = "claude:pre-tool-use"
command = "echo open"
"#;

    #[test]
    fn parse_sample() {
        let plugin = from_str(SAMPLE).expect("parse");
        assert_eq!(plugin.name, "example-plugin");
        assert_eq!(plugin.hooks.len(), 1);
    }
}
