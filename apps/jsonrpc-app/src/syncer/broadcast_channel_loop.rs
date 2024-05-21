use common_network_ids::NetworkId;
use common_sentinel::{BroadcastChannelMessages, SentinelError, SyncerBroadcastChannelMessages};

use crate::type_aliases::BroadcastChannelRx;

pub(super) async fn broadcast_channel_loop(
    network_id: NetworkId,
    mut broadcast_channel_rx: BroadcastChannelRx,
) -> Result<SyncerBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so then the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::Syncer(nid, msg)) => {
                // NOTE: We have a syncer message...
                if network_id == nid {
                    // ...and it's for this syncer so we return it
                    break 'broadcast_channel_loop Ok(msg);
                } else {
                    // ...but it's not for this syncer so we go back to listening on the receiver
                    debug!("syncer message: '{msg}' for network id: '{network_id}' ignored");
                    continue 'broadcast_channel_loop;
                }
            },
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for the syncer
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}
