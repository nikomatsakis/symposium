use agent_client_protocol::SessionNotification;

use crate::jsonrpc::{JsonRpcNotification, JsonRpcOutgoingMessage};
use crate::util::json_cast;

// Agent -> Client notifications
// These are one-way messages that agents send to clients/editors

impl JsonRpcOutgoingMessage for SessionNotification {
    fn method(&self) -> &str {
        "session/update"
    }

    fn params(&self) -> Result<Option<jsonrpcmsg::Params>, jsonrpcmsg::Error> {
        json_cast(self)
    }
}

impl JsonRpcNotification for SessionNotification {}
