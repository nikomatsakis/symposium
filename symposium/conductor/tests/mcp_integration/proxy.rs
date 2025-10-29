//! Proxy component that provides MCP tools

use acp_proxy::AcpProxyExt;
use agent_client_protocol::{self as acp};
use conductor::component::{Cleanup, ComponentProvider};
use futures::{AsyncRead, AsyncWrite};
use scp::{JsonRpcConnection, JsonRpcConnectionCx};
use std::pin::Pin;

use crate::mcp_integration::mcp_server::TestMcpServer;

pub struct ProxyComponentProvider;

impl ComponentProvider for ProxyComponentProvider {
    fn create(
        &self,
        cx: &JsonRpcConnectionCx,
        outgoing_bytes: Pin<Box<dyn AsyncWrite + Send>>,
        incoming_bytes: Pin<Box<dyn AsyncRead + Send>>,
    ) -> Result<Cleanup, acp::Error> {
        let mcp_registry = acp_proxy::McpServiceRegistry::default();
        mcp_registry.add_rmcp_server("test", TestMcpServer::new)?;

        cx.spawn(
            JsonRpcConnection::new(outgoing_bytes, incoming_bytes)
                .name("proxy-component")
                .provide_mcp(&mcp_registry)
                .proxy()
                .serve(),
        )?;

        Ok(Cleanup::None)
    }
}

pub fn create() -> Box<dyn ComponentProvider> {
    Box::new(ProxyComponentProvider)
}
