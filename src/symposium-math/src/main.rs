//! Symposium Math MCP Server
//!
//! A minimal MCP server for testing purposes. Can run standalone or as an ACP component.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod server;

#[derive(Parser)]
#[command(name = "symposium-math")]
#[command(about = "Math MCP server for testing")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run as standalone MCP server over stdio
    Mcp,
    /// Run as ACP component (proxy mode)
    Acp,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command.unwrap_or(Command::Mcp) {
        Command::Mcp => server::run_mcp_stdio().await,
        Command::Acp => server::run_acp_proxy().await,
    }
}
