use std::fmt;
use common_chain_ids::EthChainId;

#[derive(Debug, Clone)]
pub enum SyncerBroadcastChannelMessages {
    Stop,
    Start,
}

impl fmt::Display for SyncerBroadcastChannelMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Stop => "stop",
            Self::Start => "start",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    Syncer(EthChainId, SyncerBroadcastChannelMessages),
}
