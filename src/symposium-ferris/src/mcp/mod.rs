//! MCP server implementation for Ferris tools.

use std::path::PathBuf;

use sacp::mcp_server::McpServer;

use crate::Ferris;

/// Build an MCP server with the configured Ferris tools.
pub fn build_server(
    config: Ferris,
    _cwd: PathBuf,
) -> McpServer<ProxyToConductor, sacp::NullRun> {

    // Minimal compat: delegate to Ferris::into_mcp_server and return that
    config.into_mcp_server(_cwd)
}
