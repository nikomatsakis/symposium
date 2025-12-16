//! Rust Crate Sources Proxy
//!
//! An ACP proxy that provides the `rust_crate_query` MCP tool for researching
//! Rust crate sources via dedicated sub-agent sessions.

mod crate_sources_mcp;
mod eg;
mod research_agent;

use anyhow::Result;
use sacp::component::Component;
use sacp::mcp_server::McpServiceRegistry;
use sacp::schema::NewSessionRequest;
use sacp::{Agent, Client, ProxyToConductor};

/// Run the proxy as a standalone binary connected to stdio
pub async fn run() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting rust-crate-sources-proxy");

    CrateSourcesProxy.serve(sacp_tokio::Stdio::new()).await?;

    Ok(())
}

/// A proxy which forwards all messages to its successor, adding access to the rust-crate-query MCP server.
pub struct CrateSourcesProxy;

impl Component for CrateSourcesProxy {
    async fn serve(self, client: impl Component) -> Result<(), sacp::Error> {
        ProxyToConductor::builder()
            .name("rust-crate-sources-proxy")
            .on_receive_request_from(
                Client,
                async |mut request: NewSessionRequest, request_cx, cx| {
                    McpServiceRegistry::new()
                        .with_mcp_server(
                            "rust-crate-query",
                            research_agent::build_server(&request.cwd),
                        )?
                        .add_registered_mcp_servers_to(&mut request, &cx)?
                        .into_iter()
                        .for_each(|r| r.run_indefinitely());

                    cx.send_request_to(Agent, request)
                        .forward_to_request_cx(request_cx)
                },
            )
            .serve(client)
            .await
    }
}
