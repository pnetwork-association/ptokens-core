use common_eth::DefaultBlockParameter;
use ethereum_types::Address as EthAddress;
use jsonrpsee::ws_client::WsClient;

use super::eth_call;
use crate::{Challenge, ChallengeState, NetworkId, SentinelError};

pub async fn get_challenge_state(
    challenge: &Challenge,
    pnetwork_hub: &EthAddress,
    ws_client: &WsClient,
    sleep_time: u64,
    network_id: NetworkId,
) -> Result<ChallengeState, SentinelError> {
    let r = eth_call(
        pnetwork_hub,
        &ChallengeState::encode_rpc_call_data(challenge)?,
        &DefaultBlockParameter::Latest,
        ws_client,
        sleep_time,
        network_id,
    )
    .await?;

    Ok(ChallengeState::try_from(r)?)
}
