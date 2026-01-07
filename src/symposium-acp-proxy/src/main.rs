//! Symposium ACP Proxy - Main entry point
//!
//! A proxy that sits between an editor and an existing agent, enriching the
//! connection with Symposium's capabilities.
//!
//! Usage:
//!   symposium-acp-proxy --proxy sparkle --proxy ferris --proxy cargo

use anyhow::Result;
use clap::Parser;
use sacp::Component;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "symposium-acp-proxy")]
#[command(about = "Symposium ACP proxy - enriches editor-agent connections")]
struct Cli {
    /// Extension proxy to include in the chain (can be specified multiple times).
    /// Order matters - proxies are chained in the order specified.
    /// Known proxies: sparkle, ferris, cargo
    #[arg(long = "proxy", value_name = "NAME")]
    proxies: Vec<String>,

    /// Enable trace logging to the specified directory.
    /// Traces are written as timestamped .jsons files viewable with sacp-trace-viewer.
    #[arg(long)]
    trace_dir: Option<PathBuf>,

    /// Enable logging to stderr at the specified level (error, warn, info, debug, trace).
    #[arg(long)]
    log: Option<tracing::Level>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging if requested
    if let Some(level) = cli.log {
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(level.to_string()))
            .with_writer(std::io::stderr)
            .init();
    }

    if cli.proxies.is_empty() {
        anyhow::bail!(
            "No proxies specified. Use --proxy <name> to specify extensions.\n\
             Known proxies: sparkle, ferris, cargo\n\
             Example: --proxy sparkle --proxy ferris --proxy cargo"
        );
    }

    // Run Symposium as a proxy
    let mut symposium = symposium_acp_proxy::Symposium::from_proxy_names(&cli.proxies)?;

    if let Some(trace_dir) = cli.trace_dir {
        symposium = symposium.trace_dir(trace_dir);
    }

    symposium.serve(sacp_tokio::Stdio::new()).await?;

    Ok(())
}
