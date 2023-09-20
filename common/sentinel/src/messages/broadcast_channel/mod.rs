mod broadcaster;
mod rpc_server;
mod syncer;

use common_chain_ids::EthChainId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Broadcaster(BroadcasterBroadcastChannelMessages),
    Syncer(EthChainId, SyncerBroadcastChannelMessages),
}

pub use self::{
    broadcaster::BroadcasterBroadcastChannelMessages,
    rpc_server::RpcServerBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
};
