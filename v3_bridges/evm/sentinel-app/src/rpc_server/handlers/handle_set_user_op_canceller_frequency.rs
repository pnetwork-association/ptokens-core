use common_sentinel::{sanity_check_frequency, SentinelError, UserOpCancellerMessages};
use serde_json::{json, Value as Json};

use crate::{
    rpc_server::{RpcCalls, RpcParams},
    type_aliases::UserOpCancellerTx,
};

impl RpcCalls {
    pub(crate) async fn handle_set_user_op_canceller_frequency(
        params: RpcParams,
        tx: UserOpCancellerTx,
    ) -> Result<Json, SentinelError> {
        debug!("handling set user op canceller frequency rpc call...");
        let checked_params = Self::check_params(params, 1)?;
        let frequency = checked_params[0].parse::<u64>()?;
        let sanity_checked_frequency = sanity_check_frequency(frequency)?;
        let msg = UserOpCancellerMessages::SetFrequency(sanity_checked_frequency);
        tx.send(msg).await?;
        Ok(json!({"userOpCancellerUpdateFrequency": sanity_checked_frequency}))
    }
}
