mod broadcaster;
mod rpc_server;
mod status;
mod syncer;

use common_metadata::MetadataChainId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    Status(StatusBroadcastChannelMessages),
    RpcServer(RpcServerBroadcastChannelMessages),
    Broadcaster(BroadcasterBroadcastChannelMessages),
    Syncer(MetadataChainId, SyncerBroadcastChannelMessages),
}

pub use self::{
    broadcaster::BroadcasterBroadcastChannelMessages,
    rpc_server::RpcServerBroadcastChannelMessages,
    status::StatusBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
};
