use common::BridgeSide;
use common_eth::DefaultBlockParameter;
use ethereum_types::Address as EthAddress;
use jsonrpsee::ws_client::WsClient;

use super::eth_call;
use crate::{SentinelError, UserOp, UserOpSmartContractState};

pub async fn get_user_op_state(
    user_op: &UserOp,
    contract_address: &EthAddress,
    ws_client: &WsClient,
    sleep_time: u64,
    side: BridgeSide,
) -> Result<UserOpSmartContractState, SentinelError> {
    let r = eth_call(
        contract_address,
        &UserOpSmartContractState::encode_rpc_call_data(user_op)?,
        &DefaultBlockParameter::Latest,
        ws_client,
        sleep_time,
        side,
    )
    .await?;

    Ok(UserOpSmartContractState::try_from(r)?)
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use common_chain_ids::EthChainId;
    use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256};

    use super::*;
    use crate::{
        eth_rpc_calls::test_utils::get_arbitrum_protocol_queue_user_op,
        get_chain_id,
        test_utils::get_test_ws_client,
        DEFAULT_SLEEP_TIME,
    };

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_user_op_state() {
        let side = BridgeSide::default();
        let ws_client = get_test_ws_client().await;
        let target_chain_id = EthChainId::ArbitrumMainnet;
        let chain_id = EthChainId::try_from(get_chain_id(&ws_client, DEFAULT_SLEEP_TIME, side).await.unwrap()).unwrap();
        if chain_id == target_chain_id {
            let contract_address = convert_hex_to_eth_address("0xf84552a4b276b47718b8e25e8151ef749d64c4a6").unwrap();
            let op = get_arbitrum_protocol_queue_user_op();
            let uid = op.uid().unwrap();
            let expected_uid =
                convert_hex_to_h256("0x1b4d34d28d49cf03dccda141fc65118f11fb08afd28e2d2ac8520558185a71f3").unwrap();
            assert_eq!(uid, expected_uid);
            let state = get_user_op_state(&op, &contract_address, &ws_client, DEFAULT_SLEEP_TIME, side)
                .await
                .unwrap();
            let expected_state = UserOpSmartContractState::Executed;
            assert_eq!(state, expected_state);
        }
    }
}
