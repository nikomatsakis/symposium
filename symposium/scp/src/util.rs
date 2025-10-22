use agent_client_protocol as acp;

pub fn json_cast<N, M>(params: N) -> Result<M, jsonrpcmsg::Error>
where
    N: serde::Serialize,
    M: serde::de::DeserializeOwned,
{
    let json = serde_json::to_value(params).map_err(|_| jsonrpcmsg::Error::parse_error())?;
    let m = serde_json::from_value(json).map_err(|_| jsonrpcmsg::Error::parse_error())?;
    Ok(m)
}

pub fn acp_to_jsonrpc_error(err: acp::Error) -> jsonrpcmsg::Error {
    jsonrpcmsg::Error {
        code: err.code,
        message: err.message,
        data: err.data,
    }
}

pub fn jsonrpc_to_acp_error(err: jsonrpcmsg::Error) -> acp::Error {
    acp::Error {
        code: err.code,
        message: err.message,
        data: err.data,
    }
}

pub fn internal_error(err: impl ToString) -> jsonrpcmsg::Error {
    jsonrpcmsg::Error::new(-32603 /* internal error */, err.to_string())
}
