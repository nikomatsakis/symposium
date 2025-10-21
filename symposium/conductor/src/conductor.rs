use std::pin::Pin;

use futures::{AsyncRead, AsyncWrite, SinkExt, StreamExt, channel::mpsc};
use scp::{
    AcpAgentToClientMessages, AcpClientToAgentMessages, JsonRpcConnection, JsonRpcCx,
    JsonRpcNotification, JsonRpcRequest, JsonRpcRequestCx, ProxyToConductorMessages,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{debug, error, info, warn};

use crate::component::Component;

pub struct Conductor<OB: AsyncWrite, IB: AsyncRead> {
    outgoing_bytes: OB,
    incoming_bytes: IB,
    conductor_rx: mpsc::Receiver<ConductorMessage>,
    components: Vec<Component>,
}

impl<OB: AsyncWrite, IB: AsyncRead> Conductor<OB, IB> {
    pub async fn run(
        outgoing_bytes: OB,
        incoming_bytes: IB,
        mut proxies: Vec<String>,
    ) -> anyhow::Result<()> {
        if proxies.len() == 0 {
            anyhow::bail!("must have at least one component")
        }

        info!(
            component_count = proxies.len(),
            components = ?proxies,
            "Starting conductor with component chain"
        );

        proxies.reverse();
        let (conductor_tx, conductor_rx) = mpsc::channel(128 /* chosen arbitrarily */);

        tokio::task::LocalSet::new()
            .run_until(async move {
                Conductor {
                    outgoing_bytes,
                    incoming_bytes,
                    components: Default::default(),
                    conductor_rx,
                }
                .launch_proxy(proxies, conductor_tx)
                .await
            })
            .await
    }

    fn launch_proxy(
        mut self,
        mut proxies: Vec<String>,
        conductor_tx: mpsc::Sender<ConductorMessage>,
    ) -> Pin<Box<impl Future<Output = anyhow::Result<()>>>> {
        Box::pin(async move {
            let Some(next_proxy) = proxies.pop() else {
                info!("All components spawned, starting message routing");
                return self.serve(conductor_tx).await;
            };

            let component_index = self.components.len();
            let remaining = proxies.len();

            info!(
                component_index,
                component_name = %next_proxy,
                remaining_components = remaining,
                "Spawning component"
            );

            let mut child = tokio::process::Command::new(&next_proxy)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            // Take ownership of the streams (can only do this once!)
            let stdin = child.stdin.take().expect("Failed to open stdin");
            let stdout = child.stdout.take().expect("Failed to open stdout");

            debug!(
                component_index,
                component_name = %next_proxy,
                "Component process spawned, setting up JSON-RPC connection"
            );

            JsonRpcConnection::new(stdin.compat_write(), stdout.compat())
                // The proxy can send *editor* messages to use
                .on_receive(AcpAgentToClientMessages::send_to({
                    let mut conductor_tx = conductor_tx.clone();
                    async move |message| {
                        conductor_tx
                            .send(ConductorMessage::ComponentToItsClientMessage {
                                component_index,
                                message,
                            })
                            .await
                    }
                }))
                .on_receive(ProxyToConductorMessages::callback(SuccessorSendCallbacks {
                    component_index,
                    conductor_tx: conductor_tx.clone(),
                }))
                .with_client(async move |jsonrpccx| {
                    self.components.push(Component { child, jsonrpccx });
                    self.launch_proxy(proxies, conductor_tx)
                        .await
                        .map_err(scp::util::internal_error)
                })
                .await
                .map_err(|err| anyhow::anyhow!("{err:?}"))
        })
    }

    async fn serve(mut self, conductor_tx: mpsc::Sender<ConductorMessage>) -> anyhow::Result<()> {
        JsonRpcConnection::new(self.outgoing_bytes, self.incoming_bytes)
            .on_receive(AcpClientToAgentMessages::send_to({
                let mut conductor_tx_clone = conductor_tx.clone();
                async move |message| {
                    conductor_tx_clone
                        .send(ConductorMessage::ClientToAgentViaProxyChain { message })
                        .await
                }
            }))
            .with_client(async |client| {
                while let Some(message) = self.conductor_rx.next().await {
                    match message {
                        // When we receive messages from the client, forward to the first item
                        // the proxy chain.
                        ConductorMessage::ClientToAgentViaProxyChain { message } => match message {
                            scp::AcpClientToAgentMessage::Request(
                                client_request,
                                json_rpc_request_cx,
                            ) => {
                                debug!(
                                    method = client_request.method(),
                                    target = "component_0",
                                    "Routing editor request to first component"
                                );
                                send_request_and_forward_response(
                                    &self.components[0].jsonrpccx,
                                    client_request,
                                    json_rpc_request_cx,
                                    conductor_tx.clone(),
                                )
                                .await;
                            }

                            scp::AcpClientToAgentMessage::Notification(
                                client_notification,
                                _json_rpc_cx,
                            ) => {
                                debug!(
                                    method = client_notification.method(),
                                    target = "component_0",
                                    "Routing editor notification to first component"
                                );
                                self.components[0]
                                    .jsonrpccx
                                    .send_notification(client_notification)?
                            }
                        },

                        ConductorMessage::ComponentToItsClientMessage {
                            component_index,
                            message,
                        } => {
                            let its_client: &JsonRpcCx = if component_index == 0 {
                                &client
                            } else {
                                &self.components[component_index - 1].jsonrpccx
                            };

                            let target = if component_index == 0 {
                                "editor"
                            } else {
                                "predecessor_component"
                            };

                            match message {
                                scp::AcpAgentToClientMessage::Request(
                                    agent_request,
                                    json_rpc_request_cx,
                                ) => {
                                    debug!(
                                        component_index,
                                        method = agent_request.method(),
                                        target,
                                        "Routing component request to its client"
                                    );
                                    send_request_and_forward_response(
                                        its_client,
                                        agent_request,
                                        json_rpc_request_cx,
                                        conductor_tx.clone(),
                                    )
                                    .await;
                                }
                                scp::AcpAgentToClientMessage::Notification(
                                    agent_notification,
                                    _json_rpc_cx,
                                ) => {
                                    debug!(
                                        component_index,
                                        method = agent_notification.method(),
                                        target,
                                        "Routing component notification to its client"
                                    );
                                    its_client.send_notification(agent_notification)?;
                                }
                            }
                        }

                        ConductorMessage::ComponentToItsSuccessorSendRequest {
                            component_index,
                            args: scp::ToSuccessorRequest { method, params },
                            component_response_cx,
                        } => {
                            let successor_index = component_index + 1;
                            if let Some(successor_component) = self.components.get(successor_index)
                            {
                                debug!(
                                    component_index,
                                    successor_index,
                                    method = %method,
                                    "Routing _proxy/successor/request to successor component"
                                );

                                let successor_response = successor_component
                                    .jsonrpccx
                                    .send_json_request(method, params);

                                let mut conductor_tx = conductor_tx.clone();
                                tokio::task::spawn_local(async move {
                                    let v = successor_response.recv().await;
                                    if let Err(error) = component_response_cx
                                        .respond(scp::ToSuccessorResponse::from(v))
                                    {
                                        ignore_send_err(
                                            conductor_tx
                                                .send(ConductorMessage::Error { error })
                                                .await,
                                        );
                                    }
                                });
                            } else {
                                warn!(
                                    component_index,
                                    "Component requested successor but it's the last in chain"
                                );
                                component_response_cx
                                    .respond_with_error(jsonrpcmsg::Error::internal_error())?;
                            }
                        }

                        ConductorMessage::ComponentToItsSuccessorSendNotification {
                            component_index,
                            args: scp::ToSuccessorNotification { method, params },
                            component_cx,
                        } => {
                            let successor_index = component_index + 1;
                            if let Some(successor_component) = self.components.get(successor_index)
                            {
                                debug!(
                                    component_index,
                                    successor_index,
                                    method = %method,
                                    "Routing _proxy/successor/notification to successor component"
                                );
                                successor_component
                                    .jsonrpccx
                                    .send_json_notification(method, params)?
                            } else {
                                warn!(
                                    component_index,
                                    "Component sent successor notification but it's the last in chain"
                                );
                                component_cx
                                    .send_error_notification(jsonrpcmsg::Error::internal_error())?;
                            }
                        }

                        ConductorMessage::Error { error } => {
                            error!(
                                error_code = error.code,
                                error_message = %error.message,
                                "Error in spawned task"
                            );
                        }
                    };
                }
                Ok(())
            })
            .await
            .map_err(|err| anyhow::anyhow!("{err:?}"))
    }
}

fn ignore_send_err<T>(_: Result<T, mpsc::SendError>) {}

async fn send_request_and_forward_response<Req: JsonRpcRequest>(
    to_cx: &JsonRpcCx,
    req: Req,
    response_cx: JsonRpcRequestCx<Req::Response>,
    mut conductor_tx: mpsc::Sender<ConductorMessage>,
) {
    let response = to_cx.send_request(req);
    tokio::task::spawn_local(async move {
        if let Err(error) = response_cx.respond_with_result(response.recv().await) {
            ignore_send_err(conductor_tx.send(ConductorMessage::Error { error }).await);
        }
    });
}

struct SuccessorSendCallbacks {
    component_index: usize,
    conductor_tx: mpsc::Sender<ConductorMessage>,
}

impl scp::ConductorCallbacks for SuccessorSendCallbacks {
    async fn successor_send_request(
        &mut self,
        args: scp::ToSuccessorRequest<serde_json::Value>,
        response: JsonRpcRequestCx<scp::ToSuccessorResponse<serde_json::Value>>,
    ) -> Result<(), agent_client_protocol::Error> {
        self.conductor_tx
            .send(ConductorMessage::ComponentToItsSuccessorSendRequest {
                component_index: self.component_index,
                args,
                component_response_cx: response,
            })
            .await
            .map_err(agent_client_protocol::Error::into_internal_error)
    }

    async fn successor_send_notification(
        &mut self,
        args: scp::ToSuccessorNotification<serde_json::Value>,
        cx: &scp::JsonRpcCx,
    ) -> Result<(), agent_client_protocol::Error> {
        self.conductor_tx
            .send(ConductorMessage::ComponentToItsSuccessorSendNotification {
                component_index: self.component_index,
                args,
                component_cx: cx.clone(),
            })
            .await
            .map_err(agent_client_protocol::Error::into_internal_error)
    }
}

pub enum ConductorMessage {
    ClientToAgentViaProxyChain {
        message: scp::AcpClientToAgentMessage,
    },

    ComponentToItsClientMessage {
        component_index: usize,
        message: scp::AcpAgentToClientMessage,
    },

    ComponentToItsSuccessorSendRequest {
        component_index: usize,
        args: scp::ToSuccessorRequest<serde_json::Value>,
        component_response_cx: JsonRpcRequestCx<scp::ToSuccessorResponse<serde_json::Value>>,
    },

    ComponentToItsSuccessorSendNotification {
        component_index: usize,
        args: scp::ToSuccessorNotification<serde_json::Value>,
        component_cx: JsonRpcCx,
    },

    Error {
        error: jsonrpcmsg::Error,
    },
}
