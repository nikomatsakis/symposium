use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod advice_for;
mod config;
mod crate_sources;
mod git_source;
mod hook;
mod mcp;
mod plugins;
pub mod tutorial;

#[derive(Parser)]
#[command(name = "symposium", version, about = "AI the Rust Way")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the Symposium tutorial for agents and humans
    Tutorial,

    /// Run as an MCP server (stdio transport)
    Mcp,

    /// Handle a hook event (invoked by editor plugins)
    Hook {
        /// The hook event (e.g., claude:pre-tool)
        event: hook::HookEvent,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    config::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Tutorial) => {
            print!("{}", tutorial::render_cli());
            ExitCode::SUCCESS
        }
        Some(Commands::Mcp) => match mcp::serve().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("MCP server error: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Commands::Hook { event }) => hook::run(event).await,
        None => {
            println!("symposium — AI the Rust Way");
            println!();
            println!("Usage: symposium <command>");
            println!();
            println!("Commands:");
            println!("  tutorial   Show the Symposium tutorial for agents and humans");
            println!("  mcp        Run as an MCP server (stdio transport)");
            println!("  hook       Handle a hook event (invoked by editor plugins)");
            println!("  help       Show this message");
            println!();
            println!("Run `symposium <command> --help` for more information.");
            ExitCode::SUCCESS
        }
    }
}
