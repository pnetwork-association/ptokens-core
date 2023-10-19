use common_sentinel::{sanity_check_frequency, ChallengeResponderMessages, SentinelError};
use serde_json::{json, Value as Json};

use crate::{
    rpc_server::{RpcCall, RpcParams},
    type_aliases::ChallengeResponderTx,
};

impl RpcCall {
    pub(crate) async fn handle_set_challenge_responder_frequency(
        params: RpcParams,
        tx: ChallengeResponderTx,
    ) -> Result<Json, SentinelError> {
        debug!("handling set challenge responder frequency rpc call...");
        let checked_params = Self::check_params(params, 1)?;

        let frequency = checked_params[0].parse::<u64>()?;
        let sanity_checked_frequency = sanity_check_frequency(frequency)?;
        let msg = ChallengeResponderMessages::SetChallengeResponseFrequency(sanity_checked_frequency);
        tx.send(msg).await?;
        Ok(json!({"challengeResponderUpdateFrequency": sanity_checked_frequency}))
    }
}
