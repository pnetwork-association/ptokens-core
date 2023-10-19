use common_sentinel::{SentinelError, UserOpCancellerMessages};
use serde_json::{json, Value as Json};

use crate::{rpc_server::RpcCall, type_aliases::UserOpCancellerTx};

// NOTE: This RPC call will attempt to cancel ALL cancellable user ops.
impl RpcCall {
    pub(crate) async fn handle_cancel_user_ops(
        user_op_canceller_tx: UserOpCancellerTx,
        core_cxn: bool,
    ) -> Result<Json, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        user_op_canceller_tx
            .send(UserOpCancellerMessages::CancelUserOps)
            .await?;
        Ok(json!({"msg": "user op canceller instructed to cancel any cancellable user ops"}))
    }
}
