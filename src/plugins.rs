use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

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
