use agent_client_protocol::{self as acp, SessionNotification};

use crate::jsonrpc::{JsonRpcMessage, JsonRpcNotification};

// Agent -> Client notifications
// These are one-way messages that agents send to clients/editors

impl JsonRpcMessage for SessionNotification {
    fn into_untyped_message(self) -> Result<crate::UntypedMessage, acp::Error> {
        let method = self.method().to_string();
        crate::UntypedMessage::new(&method, self)
    }

    fn method(&self) -> &str {
        "session/update"
    }

    fn parse_request(
        _method: &str,
        _params: &Option<jsonrpcmsg::Params>,
    ) -> Option<Result<Self, acp::Error>> {
        // This is a notification, not a request
        None
    }

    fn parse_notification(
        method: &str,
        params: &Option<jsonrpcmsg::Params>,
    ) -> Option<Result<Self, acp::Error>> {
        if method != "session/update" {
            return None;
        }
        let params = params.as_ref()?;
        Some(crate::util::json_cast(params))
    }
}

impl JsonRpcNotification for SessionNotification {}
