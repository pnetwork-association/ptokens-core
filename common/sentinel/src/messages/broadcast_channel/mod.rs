mod broadcaster;
mod challenge_responder;
mod rpc_server;
mod status_publisher;
mod syncer;

use common_metadata::MetadataChainId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Broadcaster(BroadcasterBroadcastChannelMessages),
    Syncer(MetadataChainId, SyncerBroadcastChannelMessages),
    StatusPublisher(StatusPublisherBroadcastChannelMessages),
    ChallengeResponder(ChallengeResponderBroadcastChannelMessages),
}

pub use self::{
    broadcaster::BroadcasterBroadcastChannelMessages,
    challenge_responder::ChallengeResponderBroadcastChannelMessages,
    rpc_server::RpcServerBroadcastChannelMessages,
    status_publisher::StatusPublisherBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
};
