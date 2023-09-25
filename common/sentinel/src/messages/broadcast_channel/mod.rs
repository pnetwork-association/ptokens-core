mod broadcaster;
mod rpc_server;
mod syncer;

use common_metadata::MetadataChainId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Broadcaster(BroadcasterBroadcastChannelMessages),
    Syncer(MetadataChainId, SyncerBroadcastChannelMessages),
}

pub use self::{
    broadcaster::BroadcasterBroadcastChannelMessages,
    rpc_server::RpcServerBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
};
