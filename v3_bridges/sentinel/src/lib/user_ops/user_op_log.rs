use common::Bytes;
use common_eth::EthLog;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::{
    UserOpError,
    UserOpRouterLog,
    UserOpStateManagerLog,
    CANCELLED_USER_OP_TOPIC,
    ENQUEUED_USER_OP_TOPIC,
    EXECUTED_USER_OP_TOPIC,
    WITNESSED_USER_OP_TOPIC,
};

#[serde_as]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOpLog {
    // TODO should have the state inside it? Or the topic hash?
    pub(super) origin_block_hash: Option<EthHash>,
    pub(super) origin_transaction_hash: Option<EthHash>,
    pub(super) options_mask: EthHash,
    pub(super) nonce: U256,
    pub(super) underlying_asset_decimals: U256,
    pub(super) amount: U256,
    pub(super) underlying_asset_token_address: EthAddress,
    pub(super) origin_network_id: Option<Bytes>, // TODO use type for this!
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) destination_network_id: Bytes, // TODO use type for this!
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) underlying_asset_network_id: Bytes, // TODO use type for this!
    pub(super) destination_account: String,
    pub(super) underlying_asset_name: String,
    pub(super) underlying_asset_symbol: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) user_data: Bytes,
}

impl TryFrom<&EthLog> for UserOpLog {
    type Error = UserOpError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        if l.topics.is_empty() {
            return Err(Self::Error::NoTopics);
        } else if l.topics[0] == *WITNESSED_USER_OP_TOPIC {
            return Ok(Self::from(UserOpRouterLog::try_from(l)?));
        }

        let topics = vec![
            *CANCELLED_USER_OP_TOPIC,
            *ENQUEUED_USER_OP_TOPIC,
            *EXECUTED_USER_OP_TOPIC,
        ];

        if topics.contains(&l.topics[0]) {
            Ok(Self::from(UserOpStateManagerLog::try_from(l)?))
        } else {
            Err(UserOpError::UnrecognizedTopic(l.topics[0]))
        }
    }
}

impl From<UserOpStateManagerLog> for UserOpLog {
    fn from(l: UserOpStateManagerLog) -> Self {
        Self {
            origin_block_hash: Some(l.origin_block_hash),
            origin_transaction_hash: Some(l.origin_transaction_hash),
            options_mask: l.options_mask,
            nonce: l.nonce,
            underlying_asset_decimals: l.underlying_asset_decimals,
            amount: l.amount,
            underlying_asset_token_address: l.underlying_asset_token_address,
            origin_network_id: Some(l.origin_network_id),
            destination_network_id: l.destination_network_id,
            underlying_asset_network_id: l.underlying_asset_network_id,
            destination_account: l.destination_account,
            underlying_asset_name: l.underlying_asset_name,
            underlying_asset_symbol: l.underlying_asset_symbol,
            user_data: l.user_data,
        }
    }
}

impl From<UserOpRouterLog> for UserOpLog {
    fn from(l: UserOpRouterLog) -> Self {
        Self {
            origin_block_hash: None,
            origin_transaction_hash: None,
            options_mask: l.options_mask,
            nonce: l.nonce,
            underlying_asset_decimals: l.underlying_asset_decimals,
            amount: l.asset_amount,
            underlying_asset_token_address: l.underlying_asset_token_address,
            origin_network_id: None,
            destination_network_id: l.destination_network_id,
            underlying_asset_network_id: l.underlying_asset_network_id,
            destination_account: l.destination_account,
            underlying_asset_name: l.underlying_asset_name,
            underlying_asset_symbol: l.underlying_asset_symbol,
            user_data: l.user_data,
        }
    }
}
