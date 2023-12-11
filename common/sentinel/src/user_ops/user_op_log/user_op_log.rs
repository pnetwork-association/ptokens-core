use common::Bytes;
use common_eth::EthLog;
use derive_getters::Getters;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    user_ops::{
        UserOpError,
        UserOpProtocolLog,
        UserSendLog,
        CANCELLED_USER_OP_TOPIC,
        ENQUEUED_USER_OP_TOPIC,
        EXECUTED_USER_OP_TOPIC,
        WITNESSED_USER_OP_TOPIC,
    },
    NetworkId,
};

// NOTE: So we have to parse a user op log from one of two types of logs. First there are logs
// fired from protocol events, these contain the entire `UserOperation` structure, with all fields
// present and correct. This `Operation` gets hashed and becomes the UID for the operation.
//
// The second type of log is the event emitted when a user interatcts with the contract via
// `userSend`. This log does _not_ contain all the required fields for a user `Operation`.
//
// The missing fields are from the block & transaction the event is emitted in. These fields
// are the `Option`al ones below. They however _must_ be present in order to correctly
// encode the user operation for future interactions (querying, enqueueing, executing, cancelling
// etc). This is enforced via `Result` usage in th main `UserOp` struct.

#[serde_as]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Getters)]
pub struct UserOpLog {
    #[getter(skip)]
    pub(crate) origin_block_hash: Option<EthHash>,
    #[getter(skip)]
    pub(crate) origin_transaction_hash: Option<EthHash>,
    pub(crate) options_mask: EthHash,
    pub(crate) nonce: U256,
    pub(crate) underlying_asset_decimals: U256,
    pub(crate) asset_amount: U256,
    pub(crate) user_data_protocol_fee_asset_amount: U256,
    pub(crate) network_fee_asset_amount: U256,
    pub(crate) forward_network_fee_asset_amount: U256,
    pub(crate) underlying_asset_token_address: EthAddress,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) origin_network_id: NetworkId,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) destination_network_id: NetworkId,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) forward_destination_network_id: NetworkId,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) underlying_asset_network_id: NetworkId,
    pub(crate) origin_account: String,
    pub(crate) destination_account: String,
    pub(crate) underlying_asset_name: String,
    pub(crate) underlying_asset_symbol: String,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub(crate) user_data: Bytes,
    pub(crate) is_for_protocol: bool,
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
            options_mask: EthHash::default(),
            origin_account: String::default(),
            destination_account: String::default(),
            origin_network_id: NetworkId::default(),
            underlying_asset_name: String::default(),
            origin_block_hash: Some(EthHash::zero()),
            network_fee_asset_amount: U256::default(),
            underlying_asset_decimals: U256::default(),
            underlying_asset_symbol: String::default(),
            destination_network_id: NetworkId::default(),
            origin_transaction_hash: Some(EthHash::zero()),
            forward_network_fee_asset_amount: U256::default(),
            underlying_asset_network_id: NetworkId::default(),
            user_data_protocol_fee_asset_amount: U256::default(),
            forward_destination_network_id: NetworkId::default(),
            underlying_asset_token_address: EthAddress::default(),
        }
    }
}

impl UserOpLog {
    pub fn maybe_update_fields(&mut self, origin_block_hash: EthHash, origin_transaction_hash: EthHash) {
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
    }

    pub fn origin_block_hash(&self) -> Result<EthHash, UserOpError> {
        self.origin_block_hash
            .ok_or_else(|| UserOpError::MissingField("origin_block_hash".into()))
    }

    pub fn origin_transaction_hash(&self) -> Result<EthHash, UserOpError> {
        self.origin_transaction_hash
            .ok_or_else(|| UserOpError::MissingField("origin_transaction_hash".into()))
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
            Ok(Self::from(UserOpProtocolLog::try_from(l)?))
        } else {
            Err(UserOpError::UnrecognizedTopic(l.topics[0]))
        }
    }
}

impl From<UserOpProtocolLog> for UserOpLog {
    fn from(l: UserOpProtocolLog) -> Self {
        Self {
            nonce: l.nonce,
            user_data: l.user_data,
            asset_amount: l.asset_amount,
            options_mask: l.options_mask,
            origin_account: l.origin_account,
            is_for_protocol: l.is_for_protocol,
            origin_network_id: l.origin_network_id,
            destination_account: l.destination_account,
            origin_block_hash: Some(l.origin_block_hash),
            underlying_asset_name: l.underlying_asset_name,
            destination_network_id: l.destination_network_id,
            underlying_asset_symbol: l.underlying_asset_symbol,
            network_fee_asset_amount: l.network_fee_asset_amount,
            underlying_asset_decimals: l.underlying_asset_decimals,
            origin_transaction_hash: Some(l.origin_transaction_hash),
            underlying_asset_network_id: l.underlying_asset_network_id,
            forward_destination_network_id: l.forward_destination_network_id,
            underlying_asset_token_address: l.underlying_asset_token_address,
            forward_network_fee_asset_amount: l.forward_network_fee_asset_amount,
            user_data_protocol_fee_asset_amount: l.user_data_protocol_fee_asset_amount,
        }
    }
}

impl From<UserSendLog> for UserOpLog {
    fn from(l: UserSendLog) -> Self {
        Self {
            nonce: l.nonce,
            user_data: l.user_data,
            origin_block_hash: None,
            asset_amount: l.asset_amount,
            options_mask: l.options_mask,
            origin_transaction_hash: None,
            origin_account: l.origin_account,
            is_for_protocol: l.is_for_protocol,
            origin_network_id: l.origin_network_id,
            destination_account: l.destination_account,
            underlying_asset_name: l.underlying_asset_name,
            destination_network_id: l.destination_network_id,
            underlying_asset_symbol: l.underlying_asset_symbol,
            network_fee_asset_amount: l.network_fee_asset_amount,
            underlying_asset_decimals: l.underlying_asset_decimals,
            underlying_asset_network_id: l.underlying_asset_network_id,
            forward_destination_network_id: l.forward_destination_network_id,
            underlying_asset_token_address: l.underlying_asset_token_address,
            forward_network_fee_asset_amount: l.forward_network_fee_asset_amount,
            user_data_protocol_fee_asset_amount: l.user_data_protocol_fee_asset_amount,
        }
    }
}
