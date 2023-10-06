mod broadcaster;
mod rpc_server;
mod status_publisher;
mod syncer;

use common_metadata::MetadataChainId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    Status(StatusPublisherBroadcastChannelMessages),
    RpcServer(RpcServerBroadcastChannelMessages),
    Broadcaster(BroadcasterBroadcastChannelMessages),
    Syncer(MetadataChainId, SyncerBroadcastChannelMessages),
}

pub use self::{
    broadcaster::BroadcasterBroadcastChannelMessages,
    rpc_server::RpcServerBroadcastChannelMessages,
    status_publisher::StatusPublisherBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
};
