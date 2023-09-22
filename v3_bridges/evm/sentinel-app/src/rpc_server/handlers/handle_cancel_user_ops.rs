use common_sentinel::{BroadcasterMessages, SentinelError};
use serde_json::{json, Value as Json};

use crate::rpc_server::{constants::BroadcasterTx, RpcCall};

// NOTE: This RPC call will attempt to cancel ALL cancellable user ops.
impl RpcCall {
    pub(crate) async fn handle_cancel_user_ops(
        broadcaster_tx: BroadcasterTx,
        core_cxn: bool,
    ) -> Result<Json, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        broadcaster_tx.send(BroadcasterMessages::CancelUserOps).await?;
        Ok(json!({"msg": "broadcaster instructed to cancel any cancellable user ops"}))
    }
}
