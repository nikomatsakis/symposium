use anyhow::Result;
use sacp::mcp_server::{McpConnectionTo, McpServer};
use sacp::role;
use sacp::{ByteStreams, ConnectTo, RunWithConnectionTo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

const TOOL_DESCRIPTION: &str = "\
Use the Symposium Rust tool for guidance on Rust best practices \
and how to use dependencies of the current project. \
Execute the tool with the argument `help` to learn more.";

#[derive(Deserialize, JsonSchema)]
struct RustToolInput {
    /// The command to run (e.g., "help")
    command: String,
}

#[derive(Serialize, JsonSchema)]
struct RustToolOutput {
    output: String,
}

fn build_server() -> McpServer<role::mcp::Client, impl RunWithConnectionTo<role::mcp::Client>> {
    McpServer::builder("symposium".to_string())
        .instructions(
            "Symposium — AI the Rust Way. Use the `rust` tool for Rust development guidance.",
        )
        .tool_fn(
            "rust",
            TOOL_DESCRIPTION,
            async move |input: RustToolInput, _cx: McpConnectionTo<role::mcp::Client>| {
                let output = execute_command(&input.command);
                Ok(RustToolOutput { output })
            },
            sacp::tool_fn!(),
        )
        .build()
}

fn execute_command(command: &str) -> String {
    let command = command.trim();

    if command == "help" {
        return crate::tutorial::render_mcp();
    }

    format!("Unknown command: {command}. Use `help` to see available commands.")
}

pub async fn serve() -> Result<()> {
    let server = build_server();
    let stdio = ByteStreams::new(
        tokio::io::stdout().compat_write(),
        tokio::io::stdin().compat(),
    );
    server.connect_to(stdio).await?;
    Ok(())
}
