//! User-facing MCP service for researching Rust crates.
//!
//! Provides the `rust_crate_query` tool which allows agents to request research
//! about Rust crate source code by describing what information they need.
//! The service coordinates with research_agent to spawn sub-sessions that
//! investigate crate sources and return synthesized findings.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

/// Request to start a research session for a Rust crate
#[derive(Debug)]
pub struct ResearchRequest {
    /// Name of the Rust crate to research
    pub crate_name: String,
    /// Optional semver range (e.g., "1.0", "^1.2", "~1.2.3")
    pub crate_version: Option<String>,
    /// Research prompt describing what information is needed
    pub prompt: String,
    /// Channel to send the research findings back
    pub response_tx: oneshot::Sender<serde_json::Value>,
}

/// Parameters for the rust_crate_query tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustCrateQueryParams {
    /// Name of the Rust crate to research
    pub crate_name: String,
    /// Optional semver range (e.g., "1.0", "^1.2", "~1.2.3")
    /// Defaults to latest version if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crate_version: Option<String>,
    /// Research prompt describing what information you need about the crate.
    /// Examples:
    /// - "How do I use the derive macro for custom field names?"
    /// - "What are the signatures of all methods on tokio::runtime::Runtime?"
    /// - "Show me an example of using async-trait with associated types"
    pub prompt: String,
}

/// Output from the rust_crate_query tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct RustCrateQueryOutput {
    /// The research findings
    result: serde_json::Value,
}

/// Build the MCP server for crate research queries
pub fn build_server(research_tx: mpsc::Sender<ResearchRequest>) -> sacp_proxy::McpServer {
    use sacp_proxy::McpServer;

    McpServer::new()
        .instructions("Provides research capabilities for Rust crate source code via dedicated sub-agent sessions")
        .tool_fn(
            "rust_crate_query",
            "Research a Rust crate's source code. Provide the crate name and describe what you want to know. A specialized research agent will examine the crate sources and return findings.",
            {
                let research_tx = research_tx.clone();
                async move |input: RustCrateQueryParams, _context| {
                    let RustCrateQueryParams {
                        crate_name,
                        crate_version,
                        prompt,
                    } = input;

                    tracing::info!(
                        "Received crate query for '{}' version: {:?}",
                        crate_name,
                        crate_version
                    );
                    tracing::debug!("Research prompt: {}", prompt);

                    // Create oneshot channel for the response
                    let (response_tx, response_rx) = oneshot::channel();

                    // Send research request to background task
                    let request = ResearchRequest {
                        crate_name: crate_name.clone(),
                        crate_version,
                        prompt,
                        response_tx,
                    };

                    research_tx.send(request).await.map_err(|_| {
                        anyhow::anyhow!("Failed to send research request to background task")
                    })?;

                    tracing::debug!("Research request sent, awaiting response");

                    // Wait for the response from the research session
                    let response = response_rx.await.map_err(|_| {
                        anyhow::anyhow!("Research session closed without sending response")
                    })?;

                    tracing::info!("Research complete for '{}'", crate_name);

                    Ok(RustCrateQueryOutput { result: response })
                }
            },
            |f, args, cx| Box::pin(f(args, cx)),
        )
}
