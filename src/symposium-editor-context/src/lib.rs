//! Editor context proxy
//!
//! Injects editor state (active file, selection) into agent prompts.
//! State is read from a JSON file written by the editor extension.
//!
//! The file path is specified via the `SYMPOSIUM_EDITOR_STATE_FILE` environment variable.
//! If the variable is not set or the file doesn't exist, this proxy is a no-op passthrough.

use sacp::link::ConductorToProxy;
use sacp::schema::{ContentBlock, PromptRequest, TextContent};
use sacp::{AgentPeer, ClientPeer, Component, ProxyToConductor};
use serde::Deserialize;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Environment variable that specifies the editor state file path.
pub const STATE_FILE_ENV: &str = "SYMPOSIUM_EDITOR_STATE_FILE";

/// Maximum age of the state file before it's considered stale.
const MAX_STALENESS: Duration = Duration::from_secs(30);

/// Editor state as written by the VSCode extension.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorState {
    /// Absolute path to the currently active file.
    pub active_file: Option<String>,
    /// Language identifier (e.g., "rust", "typescript").
    pub language_id: Option<String>,
    /// Current text selection, if any.
    pub selection: Option<Selection>,
    /// Workspace folder paths.
    #[serde(default)]
    pub workspace_folders: Vec<String>,
}

/// A text selection in the editor.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    /// The selected text.
    pub text: String,
    /// 1-based start line number.
    pub start_line: u32,
    /// 1-based end line number.
    pub end_line: u32,
}

/// ACP proxy component that injects editor context into prompts.
pub struct EditorContextComponent;

impl Component<ProxyToConductor> for EditorContextComponent {
    async fn serve(self, client: impl Component<ConductorToProxy>) -> Result<(), sacp::Error> {
        let state_file = std::env::var(STATE_FILE_ENV).ok().map(PathBuf::from);

        ProxyToConductor::builder()
            .name("editor-context-proxy")
            .on_receive_request_from(
                ClientPeer,
                async move |mut req: PromptRequest, request_cx, cx| {
                    if let Some(ref path) = state_file {
                        if let Some(context_text) = read_editor_context(path) {
                            req.prompt
                                .insert(0, ContentBlock::Text(TextContent::new(context_text)));
                        }
                    }
                    cx.send_request_to(AgentPeer, req)
                        .forward_to_request_cx(request_cx)
                },
                sacp::on_receive_request!(),
            )
            .serve(client)
            .await
    }
}

/// Read editor state from the JSON file and format it as a context string.
///
/// Returns `None` if the file doesn't exist, is too old, or can't be parsed.
fn read_editor_context(path: &Path) -> Option<String> {
    // Check file freshness
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    if age > MAX_STALENESS {
        tracing::debug!("Editor state file is stale ({age:?} old), skipping");
        return None;
    }

    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("Failed to read editor state file: {e}");
            return None;
        }
    };
    let state: EditorState = match serde_json::from_str(&contents) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("Failed to parse editor state: {e}");
            return None;
        }
    };

    format_context(&state)
}

/// Format editor state into a concise context string for the agent.
fn format_context(state: &EditorState) -> Option<String> {
    let active_file = state.active_file.as_deref()?;

    // Write to String is infallible, so unwrap is fine.
    let mut ctx = String::from("<editor-context>\n");

    match &state.language_id {
        Some(lang) => writeln!(ctx, "Active file: {active_file} ({lang})").unwrap(),
        None => writeln!(ctx, "Active file: {active_file}").unwrap(),
    }

    if let Some(sel) = &state.selection {
        if !sel.text.is_empty() {
            let lang_hint = state.language_id.as_deref().unwrap_or("");
            write!(
                ctx,
                "Selected text (lines {}-{}):\n```{lang_hint}\n{}\n```\n",
                sel.start_line, sel.end_line, sel.text
            )
            .unwrap();
        }
    }

    ctx.push_str("</editor-context>");

    Some(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_context_with_selection() {
        let state = EditorState {
            active_file: Some("/project/src/main.rs".to_string()),
            language_id: Some("rust".to_string()),
            selection: Some(Selection {
                text: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                start_line: 10,
                end_line: 12,
            }),
            workspace_folders: vec!["/project".to_string()],
        };

        let context = format_context(&state).unwrap();
        assert!(context.contains("Active file: /project/src/main.rs (rust)"));
        assert!(context.contains("Selected text (lines 10-12):"));
        assert!(context.contains("```rust"));
        assert!(context.contains("fn main()"));
        assert!(context.starts_with("<editor-context>"));
        assert!(context.ends_with("</editor-context>"));
    }

    #[test]
    fn test_format_context_without_selection() {
        let state = EditorState {
            active_file: Some("/project/README.md".to_string()),
            language_id: Some("markdown".to_string()),
            selection: None,
            workspace_folders: vec![],
        };

        let context = format_context(&state).unwrap();
        assert!(context.contains("Active file: /project/README.md (markdown)"));
        assert!(!context.contains("Selected text"));
    }

    #[test]
    fn test_format_context_no_active_file() {
        let state = EditorState {
            active_file: None,
            language_id: None,
            selection: None,
            workspace_folders: vec![],
        };

        assert!(format_context(&state).is_none());
    }

    #[test]
    fn test_format_context_empty_selection() {
        let state = EditorState {
            active_file: Some("/project/lib.rs".to_string()),
            language_id: Some("rust".to_string()),
            selection: Some(Selection {
                text: String::new(),
                start_line: 5,
                end_line: 5,
            }),
            workspace_folders: vec![],
        };

        let context = format_context(&state).unwrap();
        assert!(context.contains("Active file:"));
        assert!(!context.contains("Selected text"));
    }
}
