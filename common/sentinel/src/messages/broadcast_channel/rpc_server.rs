use std::fmt;

#[derive(Debug, Clone)]
pub enum RpcServerBroadcastChannelMessages {
    CoreConnected,
    CoreDisconnected,
}

impl fmt::Display for RpcServerBroadcastChannelMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "rpc server broadcast channel message:";
        let s = match self {
            Self::CoreConnected => "core connected",
            Self::CoreDisconnected => "core disconnected",
        };
        write!(f, "{prefix} {s}")
    }
}
