use std::fmt;

use common_chain_ids::EthChainId;

// TODO Move this into dir and separate accordingly

#[derive(Debug, Clone)]
pub enum SyncerBroadcastChannelMessages {
    Stop,
    Start,
    CoreConnected,
    CoreDisconnected,
}

#[derive(Debug, Clone)]
pub enum RpcServerBroadcastChannelMessages {
    CoreConnected,
    CoreDisconnected,
}

impl fmt::Display for SyncerBroadcastChannelMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "syncer broadcast channel message:";
        let s = match self {
            Self::Stop => "stop",
            Self::Start => "start",
            Self::CoreConnected => "core connected",
            Self::CoreDisconnected => "core disconnected",
        };
        write!(f, "{prefix} {s}")
    }
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

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Syncer(EthChainId, SyncerBroadcastChannelMessages),
}
