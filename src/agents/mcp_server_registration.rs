//! MCP server registration and unregistration for all supported agents.
//!
//! Each agent has its own config format and file location. This module
//! provides per-agent `register_*` and `unregister_*` functions that
//! are called from the `Agent` methods in the parent module.
//!
//! Registration is idempotent: existing entries with correct values are
//! left untouched, while stale entries are updated in place.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::output::{Output, display_path};

use super::{load_json_or_empty, save_json};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Result of upserting a symposium MCP server entry.
enum UpsertResult {
    /// Entry already existed with correct values.
    AlreadyCorrect,
    /// Entry was newly inserted.
    Inserted,
    /// Entry existed but had stale values and was updated.
    Updated,
}

/// Ensure `container["symposium"]` has the expected command and args.
/// Inserts or updates as needed.
fn upsert_json_mcp_entry(
    container: &mut serde_json::Value,
    binary_path: &str,
) -> UpsertResult {
    let expected = json!({
        "command": binary_path,
        "args": ["mcp"]
    });

    if let Some(existing) = container.get("symposium") {
        if existing == &expected {
            return UpsertResult::AlreadyCorrect;
        }
        container["symposium"] = expected;
        UpsertResult::Updated
    } else {
        container["symposium"] = expected;
        UpsertResult::Inserted
    }
}

/// Report the upsert result to the user via Output.
fn report_upsert(result: UpsertResult, display: &str, out: &Output) -> bool {
    match result {
        UpsertResult::AlreadyCorrect => {
            out.already_ok(format!("{display}: symposium MCP server already configured"));
            false
        }
        UpsertResult::Inserted => {
            out.done(format!("{display}: added symposium MCP server"));
            true
        }
        UpsertResult::Updated => {
            out.done(format!("{display}: updated symposium MCP server"));
            true
        }
    }
}

// ---------------------------------------------------------------------------
// MCP server registration functions
// ---------------------------------------------------------------------------

/// Register symposium as an MCP server in a Claude Code / Gemini settings file.
///
/// Creates or updates the `mcpServers.symposium` entry:
///
/// ```json
/// {
///   "mcpServers": {
///     "symposium": {
///       "command": "/path/to/symposium",
///       "args": ["mcp"]
///     }
///   }
/// }
/// ```
pub(super) fn register_claude_mcp_server(settings_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    let display = display_path(settings_path);
    let mut settings = load_json_or_empty(settings_path)?;

    if !settings.is_object() {
        settings = json!({});
    }

    if settings.get("mcpServers").is_none() {
        settings["mcpServers"] = json!({});
    }

    let result = upsert_json_mcp_entry(&mut settings["mcpServers"], binary_path);
    if report_upsert(result, &display, out) {
        save_json(settings_path, &settings)?;
    }

    Ok(())
}

/// Register symposium as an MCP server in a Codex CLI config file.
///
/// Creates or updates the `[mcp_servers.symposium]` section:
///
/// ```toml
/// [mcp_servers.symposium]
/// command = "/path/to/symposium"
/// args = ["mcp"]
/// ```
pub(super) fn register_codex_mcp_server(config_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    let display = display_path(config_path);

    let content = if config_path.exists() {
        fs::read_to_string(config_path)?
    } else {
        String::new()
    };

    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .unwrap_or_else(|_| toml_edit::DocumentMut::new());

    // Ensure [mcp_servers] table exists
    if !doc.contains_key("mcp_servers") {
        doc["mcp_servers"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    let needs_write = if let Some(existing) = doc["mcp_servers"].get("symposium") {
        // Check if values are correct
        let cmd_ok = existing.get("command").and_then(|v| v.as_str()) == Some(binary_path);
        let args_ok = existing
            .get("args")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.len() == 1 && a.get(0).and_then(|v| v.as_str()) == Some("mcp"));

        if cmd_ok && args_ok {
            out.already_ok(format!("{display}: symposium MCP server already configured"));
            false
        } else {
            // Update stale entry
            let mcp = doc["mcp_servers"]["symposium"].as_table_mut().unwrap();
            mcp["command"] = toml_edit::value(binary_path);
            let mut args = toml_edit::Array::new();
            args.push("mcp");
            mcp["args"] = toml_edit::value(args);
            out.done(format!("{display}: updated symposium MCP server"));
            true
        }
    } else {
        let mut server = toml_edit::Table::new();
        server["command"] = toml_edit::value(binary_path);
        let mut args = toml_edit::Array::new();
        args.push("mcp");
        server["args"] = toml_edit::value(args);
        doc["mcp_servers"]["symposium"] = toml_edit::Item::Table(server);
        out.done(format!("{display}: added symposium MCP server"));
        true
    };

    if needs_write {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(config_path, doc.to_string())?;
    }

    Ok(())
}

/// Register symposium as an MCP server in a GitHub Copilot MCP config file.
///
/// Creates or updates the top-level `symposium` entry:
///
/// ```json
/// {
///   "symposium": {
///     "command": "/path/to/symposium",
///     "args": ["mcp"]
///   }
/// }
/// ```
pub(super) fn register_copilot_mcp_server(config_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    let mut config = load_json_or_empty(config_path)?;

    let result = upsert_json_mcp_entry(&mut config, binary_path);
    if report_upsert(result, &display, out) {
        save_json(config_path, &config)?;
    }

    Ok(())
}

/// Register symposium as an MCP server in a Gemini CLI settings file.
/// Same format as Claude Code (`mcpServers.symposium`).
pub(super) fn register_gemini_mcp_server(settings_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    register_claude_mcp_server(settings_path, binary_path, out) // Same format as Claude
}

/// Register symposium as an MCP server in a Kiro MCP config file.
/// Same structure as Claude Code (`mcpServers.symposium`).
pub(super) fn register_kiro_mcp_server(config_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    let mut config = load_json_or_empty(config_path)?;

    if !config.is_object() {
        config = json!({});
    }

    if config.get("mcpServers").is_none() {
        config["mcpServers"] = json!({});
    }

    let result = upsert_json_mcp_entry(&mut config["mcpServers"], binary_path);
    if report_upsert(result, &display, out) {
        save_json(config_path, &config)?;
    }

    Ok(())
}

/// Register symposium as an MCP server in a Goose config file.
///
/// Creates or updates the `extensions.symposium` YAML section:
///
/// ```yaml
/// extensions:
///   symposium:
///     provider: mcp
///     config:
///       command: /path/to/symposium
///       args: [mcp]
/// ```
///
/// Uses string manipulation rather than a YAML library to preserve
/// comments and formatting in the user's config.
pub(super) fn register_goose_mcp_server(config_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    let display = display_path(config_path);

    let content = if config_path.exists() {
        fs::read_to_string(config_path)?
    } else {
        String::new()
    };

    let mcp_config = format!(
        r#"
extensions:
  symposium:
    provider: mcp
    config:
      command: {}
      args: [mcp]
"#,
        binary_path
    );

    if content.contains("symposium:") {
        out.already_ok(format!("{display}: symposium MCP server already configured"));
    } else {
        let new_content = if content.trim().is_empty() {
            mcp_config.trim().to_string()
        } else if content.contains("extensions:") {
            // Insert under existing extensions section
            content.replace(
                "extensions:",
                &format!("extensions:\n  symposium:\n    provider: mcp\n    config:\n      command: {}\n      args: [mcp]", binary_path)
            )
        } else {
            format!("{}\n{}", content.trim(), mcp_config.trim())
        };

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(config_path, new_content)?;
        out.done(format!("{display}: added symposium MCP server"));
    }

    Ok(())
}

/// Register symposium as an MCP server in an OpenCode config file.
///
/// Creates or updates the `mcp.symposium` entry:
///
/// ```json
/// {
///   "mcp": {
///     "symposium": {
///       "command": "/path/to/symposium",
///       "args": ["mcp"]
///     }
///   }
/// }
/// ```
pub(super) fn register_opencode_mcp_server(config_path: &Path, binary_path: &str, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    let mut config = load_json_or_empty(config_path)?;

    if !config.is_object() {
        config = json!({});
    }

    if config.get("mcp").is_none() {
        config["mcp"] = json!({});
    }

    let result = upsert_json_mcp_entry(&mut config["mcp"], binary_path);
    if report_upsert(result, &display, out) {
        save_json(config_path, &config)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// MCP server unregistration functions
// ---------------------------------------------------------------------------

pub(super) fn unregister_claude_mcp_server(settings_path: &Path, out: &Output) -> Result<()> {
    let display = display_path(settings_path);
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = load_json_or_empty(settings_path)?;

    if let Some(mcp_servers) = settings.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
        if mcp_servers.remove("symposium").is_some() {
            save_json(settings_path, &settings)?;
            out.removed(format!("{display}: removed symposium MCP server"));
        }
    }

    Ok(())
}

pub(super) fn unregister_codex_mcp_server(config_path: &Path, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(config_path)?;
    let mut doc: toml_edit::DocumentMut = content.parse()?;

    if let Some(mcp_servers) = doc.get_mut("mcp_servers").and_then(|v| v.as_table_mut()) {
        if mcp_servers.remove("symposium").is_some() {
            fs::write(config_path, doc.to_string())?;
            out.removed(format!("{display}: removed symposium MCP server"));
        }
    }

    Ok(())
}

pub(super) fn unregister_copilot_mcp_server(config_path: &Path, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    if !config_path.exists() {
        return Ok(());
    }

    let mut config = load_json_or_empty(config_path)?;

    if config.as_object_mut().and_then(|obj| obj.remove("symposium")).is_some() {
        save_json(config_path, &config)?;
        out.removed(format!("{display}: removed symposium MCP server"));
    }

    Ok(())
}

pub(super) fn unregister_gemini_mcp_server(settings_path: &Path, out: &Output) -> Result<()> {
    unregister_claude_mcp_server(settings_path, out) // Same format as Claude
}

pub(super) fn unregister_kiro_mcp_server(config_path: &Path, out: &Output) -> Result<()> {
    unregister_claude_mcp_server(config_path, out) // Same format as Claude
}

pub(super) fn unregister_goose_mcp_server(config_path: &Path, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    if !config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(config_path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut in_symposium_section = false;
    let mut section_indent = 0;
    let mut removed = false;

    for line in lines {
        if line.trim().starts_with("symposium:") {
            // Extract the indent level of the symposium key itself
            section_indent = line.len() - line.trim_start().len();
            in_symposium_section = true;
            removed = true;
            continue;
        }
        if in_symposium_section && !line.trim().is_empty() {
            let line_indent = line.len() - line.trim_start().len();
            if line_indent <= section_indent {
                in_symposium_section = false;
            }
        }
        if !in_symposium_section {
            new_lines.push(line);
        }
    }

    if removed {
        fs::write(config_path, new_lines.join("\n"))?;
        out.removed(format!("{display}: removed symposium MCP server"));
    }

    Ok(())
}

pub(super) fn unregister_opencode_mcp_server(config_path: &Path, out: &Output) -> Result<()> {
    let display = display_path(config_path);
    if !config_path.exists() {
        return Ok(());
    }

    let mut config = load_json_or_empty(config_path)?;

    if let Some(mcp_obj) = config.get_mut("mcp").and_then(|v| v.as_object_mut()) {
        if mcp_obj.remove("symposium").is_some() {
            save_json(config_path, &config)?;
            out.removed(format!("{display}: removed symposium MCP server"));
        }
    }

    Ok(())
}
