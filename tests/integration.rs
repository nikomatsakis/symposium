mod testlib;

use expect_test::expect;
use symposium::hook::{
    HookPayload, HookSubPayload, PostToolUsePayload, PreToolUsePayload, UserPromptSubmitPayload,
};

/// Replace temp directory paths with a stable placeholder for snapshot tests.
fn normalize_paths(output: &str, ctx: &testlib::TestContext) -> String {
    let config_dir = ctx.sym.config_dir().to_string_lossy().to_string();
    output.replace(&config_dir, "$CONFIG_DIR")
}

#[tokio::test]
async fn dispatch_help() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    // Clap handles "help" as a built-in, returning a parse error with help text.
    let result = ctx.invoke(&["help"]).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    expect![[r#"
        failed to parse args: 
        Usage: symposium <COMMAND>

        Commands:
          start  Get Rust guidance and list available crate skills for the workspace
          crate  Find crate sources and guidance
          help   Print this message or the help of the given subcommand(s)

        Options:
          -h, --help  Print help
    "#]]
    .assert_eq(&err);
}

#[tokio::test]
async fn dispatch_unknown_command() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let result = ctx.invoke(&["nonsense"]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn dispatch_start() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let output = ctx.invoke(&["start"]).await.unwrap();
    let output = normalize_paths(&output, &ctx);
    expect![[r#"
        # Symposium — AI the Rust Way

        Symposium helps agents write better Rust by providing up-to-date language guidance and integration with the Rust ecosystem.


        No skills available for crates in the current dependencies."#]]
    .assert_eq(&output);
}

#[tokio::test]
async fn dispatch_crate_list_with_plugins() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let output = ctx.invoke(&["crate", "--list"]).await.unwrap();
    let output = normalize_paths(&output, &ctx);
    expect!["No skills available for crates in the current dependencies."]
    .assert_eq(&output);
}

#[tokio::test]
async fn hook_pre_tool_use_builtin_empty() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let payload = HookPayload {
        sub_payload: HookSubPayload::PreToolUse(PreToolUsePayload {
            tool_name: "Bash".to_string(),
        }),
        rest: serde_json::Map::new(),
    };
    let output = ctx.invoke_hook(&payload).await;
    assert!(output.hook_specific_output.is_none());
}

// --- PostToolUse activation recording tests ---

#[tokio::test]
async fn hook_post_tool_use_records_bash_activation() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let cwd = ctx.sym.config_dir().to_string_lossy().to_string();
    let payload = HookPayload {
        sub_payload: HookSubPayload::PostToolUse(PostToolUsePayload {
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "symposium crate tokio"}),
            tool_response: serde_json::json!({"stdout": "...", "exit_code": 0}),
            session_id: Some("s1".to_string()),
            cwd: Some(cwd),
        }),
        rest: serde_json::Map::new(),
    };
    let output = ctx.invoke_hook(&payload).await;
    assert!(output.hook_specific_output.is_none());
}

#[tokio::test]
async fn hook_post_tool_use_records_mcp_activation() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let cwd = ctx.sym.config_dir().to_string_lossy().to_string();
    let payload = HookPayload {
        sub_payload: HookSubPayload::PostToolUse(PostToolUsePayload {
            tool_name: "mcp__symposium__rust".to_string(),
            tool_input: serde_json::json!({"args": ["crate", "serde"]}),
            tool_response: serde_json::json!({"output": "..."}),
            session_id: Some("s1".to_string()),
            cwd: Some(cwd),
        }),
        rest: serde_json::Map::new(),
    };
    let output = ctx.invoke_hook(&payload).await;
    assert!(output.hook_specific_output.is_none());
}

// --- UserPromptSubmit nudge tests ---

#[tokio::test]
async fn hook_user_prompt_submit_nudges_about_available_skill() {
    // plugins0 has a standalone serde skill; workspace0 has serde as a dep.
    // The nudge fires because serde is both in the workspace and has a matching skill.
    let ctx = testlib::with_fixture(&["plugins0", "workspace0"]);
    let cwd = ctx.workspace_root.as_ref().unwrap().to_string_lossy().to_string();
    let payload = HookPayload {
        sub_payload: HookSubPayload::UserPromptSubmit(UserPromptSubmitPayload {
            prompt: "I need to use `serde`".to_string(),
            session_id: Some("s1".to_string()),
            cwd: Some(cwd),
        }),
        rest: serde_json::Map::new(),
    };
    let output = ctx.invoke_hook(&payload).await;
    let ctx_text = output
        .hook_specific_output
        .as_ref()
        .and_then(|h| h.additional_context.as_deref())
        .unwrap_or("");
    assert!(
        ctx_text.contains("serde"),
        "nudge should mention serde: {ctx_text}"
    );
    expect![[r#"
        The `serde` crate has specialized guidance available.
        To load it, run: `symposium crate serde`
    "#]]
    .assert_eq(&format!("{ctx_text}\n"));
}

#[tokio::test]
async fn hook_post_tool_use_no_session_returns_empty() {
    let ctx = testlib::with_fixture(&["plugins0"]);
    let payload = HookPayload {
        sub_payload: HookSubPayload::PostToolUse(PostToolUsePayload {
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "symposium crate tokio"}),
            tool_response: serde_json::json!({"exit_code": 0}),
            session_id: None, // no session
            cwd: Some("/tmp".to_string()),
        }),
        rest: serde_json::Map::new(),
    };
    let output = ctx.invoke_hook(&payload).await;
    assert!(output.hook_specific_output.is_none());
}

#[tokio::test]
async fn hook_activation_then_no_nudge() {
    // After activating a crate via post-tool-use, a subsequent prompt mention
    // should NOT nudge about that crate.
    let ctx = testlib::with_fixture(&["plugins0", "workspace0"]);
    let cwd = ctx.workspace_root.as_ref().unwrap().to_string_lossy().to_string();

    // First: record activation via PostToolUse
    let activate = HookPayload {
        sub_payload: HookSubPayload::PostToolUse(PostToolUsePayload {
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "symposium crate serde"}),
            tool_response: serde_json::json!({"exit_code": 0}),
            session_id: Some("s1".to_string()),
            cwd: Some(cwd.clone()),
        }),
        rest: serde_json::Map::new(),
    };
    ctx.invoke_hook(&activate).await;

    // Second: mention serde in a prompt — should not nudge since already activated
    let prompt = HookPayload {
        sub_payload: HookSubPayload::UserPromptSubmit(UserPromptSubmitPayload {
            prompt: "I need to use `serde` for serialization".to_string(),
            session_id: Some("s1".to_string()),
            cwd: Some(cwd),
        }),
        rest: serde_json::Map::new(),
    };
    let output = ctx.invoke_hook(&prompt).await;
    // No nudge because serde was already activated
    assert!(output.hook_specific_output.is_none());
}
