use std::marker::PhantomData;

use agent_client_protocol as acp;
use futures::{AsyncRead, AsyncWrite};

use crate::{
    ChainHandler, FromSuccessorNotification, FromSuccessorRequest, Handled, JsonRpcConnection,
    JsonRpcHandler, JsonRpcMessage, JsonRpcNotification, JsonRpcNotificationCx, JsonRpcRequest,
    JsonRpcRequestCx,
};

/// Extension trait for [`JsonRpcConnection`] that adds S/ACP proxy capabilities.
///
/// This trait provides methods for handling messages from downstream components (successors)
/// in the proxy chain.
pub trait JsonRpcConnectionExt<OB: AsyncWrite, IB: AsyncRead, H: JsonRpcHandler> {
    /// Adds a handler for requests received from the successor component.
    ///
    /// The provided handler will receive unwrapped ACP messages - the
    /// `_proxy/successor/receive/*` protocol wrappers are handled automatically.
    /// Your handler processes normal ACP requests and notifications as if it were
    /// a regular ACP component.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use scp::proxy::JsonRpcConnectionExt;
    /// # use scp::{JsonRpcConnection, JsonRpcHandler};
    /// # struct MyHandler;
    /// # impl JsonRpcHandler for MyHandler {}
    /// # async fn example() -> Result<(), acp::Error> {
    /// JsonRpcConnection::new(tokio::io::stdin(), tokio::io::stdout())
    ///     .on_receive_from_successor(MyHandler)
    ///     .serve()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    fn on_receive_request_from_successor<R, F>(
        self,
        op: F,
    ) -> JsonRpcConnection<OB, IB, ChainHandler<H, RequestFromSuccessorHandler<R, F>>>
    where
        R: JsonRpcRequest,
        F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>;

    /// Adds a handler for messages received from the successor component.
    ///
    /// The provided handler will receive unwrapped ACP messages - the
    /// `_proxy/successor/receive/*` protocol wrappers are handled automatically.
    /// Your handler processes normal ACP requests and notifications as if it were
    /// a regular ACP component.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use scp::proxy::JsonRpcConnectionExt;
    /// # use scp::{JsonRpcConnection, JsonRpcHandler};
    /// # struct MyHandler;
    /// # impl JsonRpcHandler for MyHandler {}
    /// # async fn example() -> Result<(), acp::Error> {
    /// JsonRpcConnection::new(tokio::io::stdin(), tokio::io::stdout())
    ///     .on_receive_from_successor(MyHandler)
    ///     .serve()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    fn on_receive_notification_from_successor<N, F>(
        self,
        op: F,
    ) -> JsonRpcConnection<OB, IB, ChainHandler<H, NotificationFromSuccessorHandler<N, F>>>
    where
        N: JsonRpcNotification,
        F: AsyncFnMut(N, JsonRpcNotificationCx) -> Result<(), acp::Error>;
}

impl<OB: AsyncWrite, IB: AsyncRead, H: JsonRpcHandler> JsonRpcConnectionExt<OB, IB, H>
    for JsonRpcConnection<OB, IB, H>
{
    fn on_receive_request_from_successor<R, F>(
        self,
        op: F,
    ) -> JsonRpcConnection<OB, IB, ChainHandler<H, RequestFromSuccessorHandler<R, F>>>
    where
        R: JsonRpcRequest,
        F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>,
    {
        self.chain_handler(RequestFromSuccessorHandler::new(op))
    }

    fn on_receive_notification_from_successor<N, F>(
        self,
        op: F,
    ) -> JsonRpcConnection<OB, IB, ChainHandler<H, NotificationFromSuccessorHandler<N, F>>>
    where
        N: JsonRpcNotification,
        F: AsyncFnMut(N, JsonRpcNotificationCx) -> Result<(), acp::Error>,
    {
        self.chain_handler(NotificationFromSuccessorHandler::new(op))
    }
}

pub struct RequestFromSuccessorHandler<R, F>
where
    R: JsonRpcRequest,
    F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>,
{
    handler: F,
    phantom: PhantomData<fn(R)>,
}

impl<R, F> RequestFromSuccessorHandler<R, F>
where
    R: JsonRpcRequest,
    F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<R, F> JsonRpcHandler for RequestFromSuccessorHandler<R, F>
where
    R: JsonRpcRequest,
    F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>,
{
    async fn handle_request(
        &mut self,
        cx: JsonRpcRequestCx<serde_json::Value>,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcRequestCx<serde_json::Value>>, agent_client_protocol::Error> {
        tracing::debug!(
            request_type = std::any::type_name::<R>(),
            method = cx.method(),
            params = ?params,
            "RequestHandler::handle_request"
        );
        match <FromSuccessorRequest<R>>::parse_request(cx.method(), params) {
            Some(Ok(req)) => {
                tracing::trace!(?req, "RequestHandler::handle_request: parse completed");
                (self.handler)(req.request, cx.cast()).await?;
                Ok(Handled::Yes)
            }
            Some(Err(err)) => {
                tracing::trace!(?err, "RequestHandler::handle_request: parse errored");
                Err(err)
            }
            None => {
                tracing::trace!("RequestHandler::handle_request: parse failed");
                Ok(Handled::No(cx))
            }
        }
    }

    fn describe_chain(&self) -> impl std::fmt::Debug {
        std::any::type_name::<R>()
    }
}

pub struct NotificationFromSuccessorHandler<N, F>
where
    N: JsonRpcNotification,
    F: AsyncFnMut(N, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    handler: F,
    phantom: PhantomData<fn(N)>,
}

impl<N, F> NotificationFromSuccessorHandler<N, F>
where
    N: JsonRpcNotification,
    F: AsyncFnMut(N, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<N, F> JsonRpcHandler for NotificationFromSuccessorHandler<N, F>
where
    N: JsonRpcNotification,
    F: AsyncFnMut(N, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    async fn handle_notification(
        &mut self,
        cx: JsonRpcNotificationCx,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcNotificationCx>, agent_client_protocol::Error> {
        tracing::debug!(
            request_type = std::any::type_name::<N>(),
            method = cx.method(),
            params = ?params,
            "NotificationFromSuccessorHandler::handle_request"
        );
        match <FromSuccessorNotification<N>>::parse_notification(cx.method(), params) {
            Some(Ok(req)) => {
                tracing::trace!(
                    ?req,
                    "NotificationFromSuccessorHandler::handle_request: parse completed"
                );
                (self.handler)(req.notification, cx).await?;
                Ok(Handled::Yes)
            }
            Some(Err(err)) => {
                tracing::trace!(
                    ?err,
                    "NotificationFromSuccessorHandler::handle_request: parse errored"
                );
                Err(err)
            }
            None => {
                tracing::trace!("NotificationFromSuccessorHandler::handle_request: parse failed");
                Ok(Handled::No(cx))
            }
        }
    }

    fn describe_chain(&self) -> impl std::fmt::Debug {
        std::any::type_name::<N>()
    }
}
