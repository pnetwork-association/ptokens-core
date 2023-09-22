use common::Bytes;
use common_eth::EthLog;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::{
    UserOpError,
    UserOpPnetworkHubLog,
    UserSendLog,
    CANCELLED_USER_OP_TOPIC,
    ENQUEUED_USER_OP_TOPIC,
    EXECUTED_USER_OP_TOPIC,
    WITNESSED_USER_OP_TOPIC,
};

#[serde_as]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserOpLog {
    // TODO should have the state inside it? Or the topic hash?
    pub(super) origin_block_hash: Option<EthHash>,
    pub(super) origin_transaction_hash: Option<EthHash>,
    pub(super) options_mask: EthHash,
    pub(super) nonce: U256,
    pub(super) underlying_asset_decimals: U256,
    pub(super) asset_amount: U256,
    pub(super) protocol_fee_asset_amount: U256,
    pub(super) network_fee_asset_amount: U256,
    pub(super) forward_network_fee_asset_amount: U256,
    pub(super) underlying_asset_token_address: EthAddress,
    pub(super) origin_network_id: Option<Bytes>, // TODO use type for this!
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) destination_network_id: Bytes, // TODO use type for this!
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) forward_destination_network_id: Bytes, // TODO use type for this!
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) underlying_asset_network_id: Bytes, // TODO use type for this!
    pub(super) origin_account: String,
    pub(super) destination_account: String,
    pub(super) underlying_asset_name: String,
    pub(super) underlying_asset_symbol: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(super) user_data: Bytes,
    pub(super) is_for_protocol: bool,
}

impl Default for UserOpLog {
    fn default() -> Self {
        Self {
            // NOTE The optional fields here cannot be missing. They're only optional due to parsing
            // from logs where in the case of a WITNESSED event, the log does NOT contain them.
            user_data: vec![],
            nonce: U256::default(),
            is_for_protocol: false,
            asset_amount: U256::default(),
            destination_network_id: vec![],
            origin_network_id: Some(vec![]),
            options_mask: EthHash::default(),
            origin_account: String::default(),
            underlying_asset_network_id: vec![],
            forward_destination_network_id: vec![],
            destination_account: String::default(),
            underlying_asset_name: String::default(),
            origin_block_hash: Some(EthHash::zero()),
            network_fee_asset_amount: U256::default(),
            underlying_asset_decimals: U256::default(),
            underlying_asset_symbol: String::default(),
            protocol_fee_asset_amount: U256::default(),
            origin_transaction_hash: Some(EthHash::zero()),
            forward_network_fee_asset_amount: U256::default(),
            underlying_asset_token_address: EthAddress::default(),
        }
    }
}

impl UserOpLog {
    pub fn maybe_update_fields(
        &mut self,
        origin_block_hash: EthHash,
        origin_transaction_hash: EthHash,
        origin_network_id: Bytes,
    ) {
        // NOTE: A witnessed user op needs these fields from the block it was witnessed in. All
        // other states will include the full log, with these fields already included.
        if self.origin_block_hash.is_none() {
            debug!("updating `origin_block_hash` in `UserOpLog`");
            self.origin_block_hash = Some(origin_block_hash)
        };

        if self.origin_transaction_hash.is_none() {
            debug!("updating `origin_transaction_hash` in `UserOpLog`");
            self.origin_transaction_hash = Some(origin_transaction_hash)
        };

        if self.origin_network_id.is_none() {
            debug!("updating `origin_network_id` in `UserOpLog`");
            self.origin_network_id = Some(origin_network_id)
        };
    }

    pub fn origin_block_hash(&self) -> Result<EthHash, UserOpError> {
        self.origin_block_hash
            .ok_or_else(|| UserOpError::MissingField("origin_block_hash".into()))
    }

    pub fn origin_transaction_hash(&self) -> Result<EthHash, UserOpError> {
        self.origin_transaction_hash
            .ok_or_else(|| UserOpError::MissingField("origin_transaction_hash".into()))
    }

    pub fn origin_network_id(&self) -> Result<Bytes, UserOpError> {
        self.origin_network_id
            .clone()
            .ok_or_else(|| UserOpError::MissingField("origin_network_id".into()))
    }
}

impl TryFrom<&EthLog> for UserOpLog {
    type Error = UserOpError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        if l.topics.is_empty() {
            return Err(Self::Error::NoTopics);
        } else if l.topics[0] == *WITNESSED_USER_OP_TOPIC {
            return Ok(Self::from(UserSendLog::try_from(l)?));
        }

        let topics = [
            *CANCELLED_USER_OP_TOPIC,
            *ENQUEUED_USER_OP_TOPIC,
            *EXECUTED_USER_OP_TOPIC,
        ];

        if topics.contains(&l.topics[0]) {
            Ok(Self::from(UserOpPnetworkHubLog::try_from(l)?))
        } else {
            Err(UserOpError::UnrecognizedTopic(l.topics[0]))
        }
    }
}

impl From<UserOpPnetworkHubLog> for UserOpLog {
    fn from(l: UserOpPnetworkHubLog) -> Self {
        Self {
            nonce: l.nonce,
            user_data: l.user_data,
            asset_amount: l.asset_amount,
            options_mask: l.options_mask,
            origin_account: l.origin_account,
            is_for_protocol: l.is_for_protocol,
            destination_account: l.destination_account,
            origin_network_id: Some(l.origin_network_id),
            origin_block_hash: Some(l.origin_block_hash),
            underlying_asset_name: l.underlying_asset_name,
            destination_network_id: l.destination_network_id,
            underlying_asset_symbol: l.underlying_asset_symbol,
            network_fee_asset_amount: l.network_fee_asset_amount,
            underlying_asset_decimals: l.underlying_asset_decimals,
            protocol_fee_asset_amount: l.protocol_fee_asset_amount,
            origin_transaction_hash: Some(l.origin_transaction_hash),
            underlying_asset_network_id: l.underlying_asset_network_id,
            forward_destination_network_id: l.forward_destination_network_id,
            underlying_asset_token_address: l.underlying_asset_token_address,
            forward_network_fee_asset_amount: l.forward_network_fee_asset_amount,
        }
    }
}

impl From<UserSendLog> for UserOpLog {
    fn from(l: UserSendLog) -> Self {
        Self {
            nonce: l.nonce,
            user_data: l.user_data,
            origin_network_id: None,
            origin_block_hash: None,
            asset_amount: l.asset_amount,
            options_mask: l.options_mask,
            origin_transaction_hash: None,
            origin_account: l.origin_account,
            is_for_protocol: l.is_for_protocol,
            destination_account: l.destination_account,
            underlying_asset_name: l.underlying_asset_name,
            destination_network_id: l.destination_network_id,
            underlying_asset_symbol: l.underlying_asset_symbol,
            network_fee_asset_amount: l.network_fee_asset_amount,
            underlying_asset_decimals: l.underlying_asset_decimals,
            protocol_fee_asset_amount: l.protocol_fee_asset_amount,
            underlying_asset_network_id: l.underlying_asset_network_id,
            forward_destination_network_id: l.forward_destination_network_id,
            underlying_asset_token_address: l.underlying_asset_token_address,
            forward_network_fee_asset_amount: l.forward_network_fee_asset_amount,
        }
    }
}
