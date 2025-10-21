use crate::{
    jsonrpc::{self, Handled, JsonRpcCx, JsonRpcHandler, JsonRpcRequestCx},
    proxy::{ToSuccessorNotification, ToSuccessorRequest, ToSuccessorResponse},
    util::{acp_to_jsonrpc_error, json_cast},
};
use agent_client_protocol as acp;

/// Callbacks for the conductor who receives requests from proxies to forward messages over to their successor.
pub trait ConductorCallbacks {
    /// Name of the method to be invoked
    /// Parameters for the method invocation
    async fn successor_send_request(
        &mut self,
        args: ToSuccessorRequest<serde_json::Value>,
        response: JsonRpcRequestCx<ToSuccessorResponse<serde_json::Value>>,
    ) -> Result<(), acp::Error>;

    /// Name of the method to be invoked
    /// Parameters for the method invocation
    async fn successor_send_notification(
        &mut self,
        args: ToSuccessorNotification<serde_json::Value>,
        cx: &JsonRpcCx,
    ) -> Result<(), acp::Error>;
}

/// Message handler for messages targeting the conductor.
pub struct AcpConductorMessages<CB: ConductorCallbacks> {
    callbacks: CB,
}

impl<CB: ConductorCallbacks> AcpConductorMessages<CB> {
    /// Create new handler that invokes `callbacks` when requests from proxies are received.
    pub fn new(callbacks: CB) -> Self {
        Self { callbacks }
    }
}

impl<CB: ConductorCallbacks> JsonRpcHandler for AcpConductorMessages<CB> {
    async fn handle_request(
        &mut self,
        method: &str,
        params: &Option<jsonrpcmsg::Params>,
        response: JsonRpcRequestCx<serde_json::Value>,
    ) -> Result<crate::jsonrpc::Handled<JsonRpcRequestCx<serde_json::Value>>, jsonrpcmsg::Error>
    {
        match method {
            "_proxy/successor/send/request" => {
                // Proxy is requesting us to send this message to their successor.
                self.callbacks
                    .successor_send_request(json_cast(params)?, response.cast())
                    .await
                    .map_err(acp_to_jsonrpc_error)?;
                Ok(Handled::Yes)
            }

            _ => Ok(Handled::No(response)),
        }
    }

    async fn handle_notification(
        &mut self,
        method: &str,
        params: &Option<jsonrpcmsg::Params>,
        cx: &JsonRpcCx,
    ) -> Result<crate::jsonrpc::Handled<()>, jsonrpcmsg::Error> {
        match method {
            "_proxy/successor/send/notification" => {
                // Proxy is requesting us to send this message to their successor.
                self.callbacks
                    .successor_send_notification(json_cast(params)?, cx)
                    .await
                    .map_err(acp_to_jsonrpc_error)?;
                Ok(Handled::Yes)
            }

            _ => Ok(Handled::No(())),
        }
    }
}
