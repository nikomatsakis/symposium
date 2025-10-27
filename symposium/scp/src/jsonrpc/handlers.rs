use crate::jsonrpc::{Handled, JsonRpcHandler};
use crate::{JsonRpcNotification, JsonRpcNotificationCx, JsonRpcRequest, UntypedMessage};
use agent_client_protocol as acp;
use std::marker::PhantomData;
use std::ops::AsyncFnMut;

use super::JsonRpcRequestCx;

/// Null handler that accepts no messages.
#[derive(Default)]
pub struct NullHandler {}

impl JsonRpcHandler for NullHandler {}

pub struct RequestHandler<R, F>
where
    R: JsonRpcRequest,
    F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>,
{
    handler: F,
    phantom: PhantomData<fn(R)>,
}

impl<R, F> RequestHandler<R, F>
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

impl<R, F> JsonRpcHandler for RequestHandler<R, F>
where
    R: JsonRpcRequest,
    F: AsyncFnMut(R, JsonRpcRequestCx<R::Response>) -> Result<(), acp::Error>,
{
    async fn handle_request(
        &mut self,
        cx: JsonRpcRequestCx<serde_json::Value>,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcRequestCx<serde_json::Value>>, agent_client_protocol::Error> {
        match R::parse_request(cx.method(), params) {
            Some(Ok(req)) => {
                (self.handler)(req, cx.cast()).await?;
                Ok(Handled::Yes)
            }
            Some(Err(err)) => Err(err),
            None => Ok(Handled::No(cx)),
        }
    }
}

pub struct NotificationHandler<N, F>
where
    N: JsonRpcNotification,
    F: AsyncFnMut(N, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    handler: F,
    phantom: PhantomData<fn(N)>,
}

impl<R, F> NotificationHandler<R, F>
where
    R: JsonRpcNotification,
    F: AsyncFnMut(R, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<R, F> JsonRpcHandler for NotificationHandler<R, F>
where
    R: JsonRpcNotification,
    F: AsyncFnMut(R, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    async fn handle_notification(
        &mut self,
        cx: JsonRpcNotificationCx,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcNotificationCx>, agent_client_protocol::Error> {
        match R::parse_notification(cx.method(), params) {
            Some(Ok(req)) => {
                (self.handler)(req, cx).await?;
                Ok(Handled::Yes)
            }
            Some(Err(err)) => Err(err),
            None => Ok(Handled::No(cx)),
        }
    }
}

/// Handler that tries H1 and then H2.
pub struct ChainHandler<H1, H2>
where
    H1: JsonRpcHandler,
    H2: JsonRpcHandler,
{
    handler1: H1,
    handler2: H2,
}

impl<H1, H2> ChainHandler<H1, H2>
where
    H1: JsonRpcHandler,
    H2: JsonRpcHandler,
{
    pub fn new(handler1: H1, handler2: H2) -> Self {
        Self { handler1, handler2 }
    }
}

impl<H1, H2> JsonRpcHandler for ChainHandler<H1, H2>
where
    H1: JsonRpcHandler,
    H2: JsonRpcHandler,
{
    async fn handle_request(
        &mut self,
        cx: JsonRpcRequestCx<serde_json::Value>,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcRequestCx<serde_json::Value>>, acp::Error> {
        match self.handler1.handle_request(cx, params).await? {
            Handled::Yes => Ok(Handled::Yes),
            Handled::No(cx) => self.handler2.handle_request(cx, params).await,
        }
    }

    async fn handle_notification(
        &mut self,
        cx: JsonRpcNotificationCx,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcNotificationCx>, acp::Error> {
        match self.handler1.handle_notification(cx, params).await? {
            Handled::Yes => Ok(Handled::Yes),
            Handled::No(cx) => self.handler2.handle_notification(cx, params).await,
        }
    }
}

/// Generic JSON-RPC handler that provides callbacks for incoming requests and notifications.
pub struct AllMessages<HandleReq, HandleNotification>
where
    HandleReq:
        AsyncFnMut(UntypedMessage, JsonRpcRequestCx<serde_json::Value>) -> Result<(), acp::Error>,
    HandleNotification: AsyncFnMut(UntypedMessage, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    handle_request: HandleReq,
    handle_notification: HandleNotification,
}

impl<HandleReq, HandleNotification> AllMessages<HandleReq, HandleNotification>
where
    HandleReq:
        AsyncFnMut(UntypedMessage, JsonRpcRequestCx<serde_json::Value>) -> Result<(), acp::Error>,
    HandleNotification: AsyncFnMut(UntypedMessage, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    /// Create a handler that forwards all requests to the given callback.
    ///
    /// The callback receives:
    /// - `method`: The JSON-RPC method name
    /// - `params`: The JSON-RPC parameters (if any)
    /// - `response_cx`: Context for sending the response
    ///
    /// Example usage:
    /// ```ignore
    /// connection
    ///     .on_receive(GenericHandler::send_to(|method, params, response_cx| async move {
    ///         // Forward to mpsc channel
    ///         tx.send((method, params, response_cx)).await?;
    ///         Ok(())
    ///     }))
    ///     .serve()
    ///     .await
    /// ```
    pub fn call(handle_request: HandleReq, handle_notification: HandleNotification) -> Self {
        Self {
            handle_request,
            handle_notification,
        }
    }
}

impl<HandleReq, HandleNotification> JsonRpcHandler for AllMessages<HandleReq, HandleNotification>
where
    HandleReq:
        AsyncFnMut(UntypedMessage, JsonRpcRequestCx<serde_json::Value>) -> Result<(), acp::Error>,
    HandleNotification: AsyncFnMut(UntypedMessage, JsonRpcNotificationCx) -> Result<(), acp::Error>,
{
    async fn handle_request(
        &mut self,
        cx: JsonRpcRequestCx<serde_json::Value>,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcRequestCx<serde_json::Value>>, agent_client_protocol::Error> {
        let message = UntypedMessage::new(cx.method(), params)?;
        (self.handle_request)(message, cx).await?;
        Ok(Handled::Yes)
    }

    async fn handle_notification(
        &mut self,
        cx: JsonRpcNotificationCx,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Result<Handled<JsonRpcNotificationCx>, agent_client_protocol::Error> {
        let message = UntypedMessage::new(cx.method(), params)?;
        (self.handle_notification)(message, cx).await?;
        Ok(Handled::Yes)
    }
}
