use std::pin::Pin;

use jsonrpcmsg::Params;
use scp::jsonrpc::{JsonRpcConnection, JsonRpcRequestCx};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::component::Component;

pub struct Conductor {
    components: Vec<Component>,
}

impl Conductor {
    pub async fn run(mut proxies: Vec<String>) -> anyhow::Result<()> {
        proxies.reverse();
        Conductor {
            components: Default::default(),
        }
        .launch_proxy(proxies)
        .await
    }

    fn launch_proxy(
        mut self,
        mut proxies: Vec<String>,
    ) -> Pin<Box<impl Future<Output = anyhow::Result<()>>>> {
        Box::pin(async move {
            let Some(next_proxy) = proxies.pop() else {
                return self.serve().await;
            };

            let mut child = tokio::process::Command::new(next_proxy)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            // Take ownership of the streams (can only do this once!)
            let stdin = child.stdin.take().expect("Failed to open stdin");
            let stdout = child.stdout.take().expect("Failed to open stdout");

            JsonRpcConnection::new(stdin.compat_write(), stdout.compat())
                .with_client(async move |jsonrpccx| {
                    self.components.push(Component { child, jsonrpccx });
                    self.launch_proxy(proxies)
                        .await
                        .map_err(scp::util::internal_error)
                })
                .await
                .map_err(|err| anyhow::anyhow!("{err:?}"))
        })
    }

    async fn serve(self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub enum ConductorMessage {
    Initialize {
        args: agent_client_protocol::InitializeRequest,
        response: JsonRpcRequestCx<agent_client_protocol::InitializeResponse>,
    },

    ProxyToSuccessor {
        index: usize,
        method: String,
        params: Option<Params>,
        response: JsonRpcRequestCx<serde_json::Value>,
    },
}
