use agent_client_protocol::CancelNotification;

use crate::jsonrpc::{JsonRpcNotification, JsonRpcOutgoingMessage};
use crate::util::json_cast;

impl JsonRpcOutgoingMessage for CancelNotification {
    fn method(&self) -> &str {
        "session/cancel"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcNotification for CancelNotification {}
