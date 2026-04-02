use std::io::{Read, Write};
use std::process::{Command, ExitCode, Stdio};

use serde::{Deserialize, Serialize};

use crate::plugins::ParsedPlugin;

#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
pub enum HookEvent {
    #[value(name = "pre-tool-use")]
    #[serde(rename = "PreToolUse")]
    PreToolUse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayload {
    #[serde(flatten)]
    pub sub_payload: HookSubPayload,
    #[serde(flatten)]
    pub rest: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name")]
pub enum HookSubPayload {
    #[serde(rename = "PreToolUse")]
    PreToolUse(PreToolUsePayload),
}

impl HookSubPayload {
    pub fn hook_event(&self) -> HookEvent {
        match self {
            HookSubPayload::PreToolUse(_) => HookEvent::PreToolUse,
        }
    }

    #[tracing::instrument(ret)]
    pub fn matches_matcher(&self, matcher: &str) -> bool {
        // TODO: I'm not sure what exactly Claude's rules are, but this is fine for now
        if matcher == "*" {
            return true;
        }
        match self {
            HookSubPayload::PreToolUse(payload) => matcher.contains(&payload.tool_name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUsePayload {
    pub tool_name: String,
}

pub async fn run(event: HookEvent) -> ExitCode {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        tracing::warn!(?event, error = %e, "failed to read hook stdin");
        return ExitCode::SUCCESS;
    }

    let payload = serde_json::from_str::<HookPayload>(&input);
    let Ok(payload) = payload else {
        tracing::warn!(
            ?event,
            error = "invalid hook payload",
            "failed to parse hook stdin as JSON"
        );
        return ExitCode::FAILURE;
    };

    if payload.sub_payload.hook_event() != event {
        tracing::warn!(?event, payload_event = ?payload.sub_payload.hook_event(), "hook event mismatch between CLI arg and payload");
        return ExitCode::FAILURE;
    }

    dispatch_hook(payload).await
}

/// Handle hook dispatch for a parsed payload string. Separated from `run`
/// so tests and other callers can invoke it without wiring stdin.
pub async fn dispatch_hook(payload: HookPayload) -> ExitCode {
    tracing::info!(?payload, "hook invoked");

    let plugins = crate::plugins::load_all_plugins();
    let hooks = hooks_for_payload(&plugins, &payload);

    for (plugin_name, hook) in hooks {
        tracing::info!(?plugin_name, hook = %hook.name, cmd = %hook.command, "running plugin hook");
        let spawn_res = Command::new("sh")
            .arg("-c")
            .arg(&hook.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();

        match spawn_res {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) =
                        stdin.write_all(serde_json::to_string(&payload).unwrap().as_bytes())
                    {
                        tracing::warn!(error = %e, "failed to write hook stdin");
                    }
                }

                match child.wait() {
                    Ok(status) => tracing::info!(?status, "hook finished"),
                    Err(e) => tracing::warn!(error = %e, "failed waiting for hook process"),
                }
            }
            Err(e) => tracing::warn!(error = %e, "failed to spawn hook command"),
        }
    }

    ExitCode::SUCCESS
}

/// Return all hooks (with their plugin name) that match the event in `payload`.
fn hooks_for_payload(
    plugins: &[crate::plugins::ParsedPlugin],
    payload: &HookPayload,
) -> Vec<(String, crate::plugins::Hook)> {
    tracing::debug!(?payload);

    let mut out = Vec::new();

    for ParsedPlugin { path: _, plugin } in plugins {
        let name = plugin.name.clone();
        for hook in &plugin.hooks {
            tracing::debug!(?hook);
            if hook.event != payload.sub_payload.hook_event() {
                continue;
            }
            if let Some(matcher) = &hook.matcher {
                if !payload.sub_payload.matches_matcher(matcher) {
                    tracing::info!(
                        ?payload,
                        ?matcher,
                        "skipping hook due to non-matching matcher"
                    );
                    continue;
                }
            }
            out.push((name.clone(), hook.clone()));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use indoc::formatdoc;

    fn setup_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .compact()
            .with_ansi(false)
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::DEBUG.into()),
            )
            .try_init();
    }

    #[tokio::test]
    async fn plugin_hooks_run_and_create_files() {
        setup_tracing();

        let tmp = tempfile::tempdir().expect("tempdir");
        let home = tmp.path();

        // Point HOME to our temp dir so plugins_dir() is under it.
        unsafe {
            std::env::set_var("HOME", home);
        }

        // Ensure plugins dir exists and get its path.
        let plugins_dir = crate::config::plugins_dir();

        // Prepare two output files that the hooks will create.
        let out1 = home.join("out1.txt");
        let out2 = home.join("out2.txt");
        let out3 = home.join("out3.txt");
        let out4 = home.join("out4.txt");
        let out5 = home.join("out5.txt");

        // Create two plugin TOML files that run simple echo commands.
        let p1 = formatdoc! {r#"
            name = "plugin-one"

            [[hooks]]
            name = "write1"
            event = "PreToolUse"
            command = "sh -c 'echo plugin-one-write1 > {out1}'"
        "#, out1 = out1.display()};

        let p2 = formatdoc! {r#"
            name = "plugin-two"

            [[hooks]]
            name = "write2"
            event = "PreToolUse"
            matcher = "*"
            command = "sh -c 'echo plugin-two-write2 > {out2}'"

            [[hooks]]
            name = "write3"
            event = "PreToolUse"
            matcher = "Bash"
            command = "sh -c 'echo plugin-two-write3 > {out3}'"

            [[hooks]]
            name = "write4"
            event = "PreToolUse"
            matcher = "Bash|Read"
            command = "sh -c 'echo plugin-two-write4 > {out4}'"

            [[hooks]]
            name = "write4"
            event = "PreToolUse"
            matcher = "Read|Write"
            command = "sh -c 'echo plugin-two-write5 > {out5}'"
        "#,
            out2 = out2.display(),
            out3 = out3.display(),
            out4 = out4.display(),
            out5 = out5.display(),
        };

        fs::write(plugins_dir.join("plugin-one.toml"), p1).expect("write plugin1");
        fs::write(plugins_dir.join("plugin-two.toml"), p2).expect("write plugin2");

        // Run the hook event. This will spawn the commands which create the files.
        let payload = HookPayload {
            sub_payload: HookSubPayload::PreToolUse(PreToolUsePayload {
                tool_name: "Bash".to_string(),
            }),
            rest: serde_json::Map::new(),
        };
        let _ = dispatch_hook(payload).await;

        // Verify files were created and contain expected contents.
        let got1 = fs::read_to_string(&out1).expect("read out1");
        let got2 = fs::read_to_string(&out2).expect("read out2");
        let got3 = fs::read_to_string(&out3).expect("read out3");
        let got4 = fs::read_to_string(&out4).expect("read out4");

        assert!(got1.contains("plugin-one-write1"));
        assert!(got2.contains("plugin-two-write2"));
        assert!(got3.contains("plugin-two-write3"));
        assert!(got4.contains("plugin-two-write4"));

        // No file created, matcher doesn't match
        assert!(fs::read_to_string(&out5).is_err());
    }
}
