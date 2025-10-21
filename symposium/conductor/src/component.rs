use scp::JsonRpcCx;
use tokio::process::Child;

pub struct Component {
    #[expect(dead_code)]
    pub child: Child,
    pub jsonrpccx: JsonRpcCx,
}
