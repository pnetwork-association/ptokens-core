mod challenge_responder;
mod rpc_server;
mod status_publisher;
mod syncer;
mod user_op_canceller;

use common_metadata::MetadataChainId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Syncer(MetadataChainId, SyncerBroadcastChannelMessages),
    StatusPublisher(StatusPublisherBroadcastChannelMessages),
    UserOpCanceller(UserOpCancellerBroadcastChannelMessages),
    ChallengeResponder(ChallengeResponderBroadcastChannelMessages),
}

pub use self::{
    challenge_responder::ChallengeResponderBroadcastChannelMessages,
    rpc_server::RpcServerBroadcastChannelMessages,
    status_publisher::StatusPublisherBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
    user_op_canceller::UserOpCancellerBroadcastChannelMessages,
};
