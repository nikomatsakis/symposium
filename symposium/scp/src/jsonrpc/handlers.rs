use crate::jsonrpc::{Handled, JsonRpcHandler};
use crate::{JsonRpcNotification, JsonRpcNotificationCx, JsonRpcRequest};
use agent_client_protocol as acp;
use std::marker::PhantomData;
use std::ops::AsyncFnMut;

use super::JsonRpcRequestCx;

/// Null handler that accepts no messages.
#[derive(Default)]
pub struct NullHandler {}

impl JsonRpcHandler for NullHandler {
    fn describe_chain(&self) -> impl std::fmt::Debug {
        "(null)"
    }
}

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
        tracing::debug!(
            request_type = std::any::type_name::<R>(),
            method = cx.method(),
            params = ?params,
            "RequestHandler::handle_request"
        );
        match R::parse_request(cx.method(), params) {
            Some(Ok(req)) => {
                tracing::trace!(?req, "RequestHandler::handle_request: parse completed");
                (self.handler)(req, cx.cast()).await?;
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
        tracing::debug!(
            type_name = std::any::type_name::<R>(),
            method = cx.method(),
            params = ?params,
            "handle_notification"
        );
        match R::parse_notification(cx.method(), params) {
            Some(Ok(req)) => {
                tracing::trace!(
                    ?req,
                    "NotificationHandler::handle_notification: parse completed"
                );
                (self.handler)(req, cx).await?;
                Ok(Handled::Yes)
            }
            Some(Err(err)) => {
                tracing::trace!(
                    ?err,
                    "NotificationHandler::handle_notification: parse errored"
                );
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

    fn describe_chain(&self) -> impl std::fmt::Debug {
        return DebugImpl {
            handler1: &self.handler1,
            handler2: &self.handler2,
        };

        struct DebugImpl<'h, H1, H2> {
            handler1: &'h H1,
            handler2: &'h H2,
        }

        impl<H1: JsonRpcHandler, H2: JsonRpcHandler> std::fmt::Debug for DebugImpl<'_, H1, H2> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{:?}, {:?}",
                    self.handler1.describe_chain(),
                    self.handler2.describe_chain()
                )
            }
        }
    }
}
