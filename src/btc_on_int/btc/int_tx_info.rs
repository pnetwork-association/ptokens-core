use bitcoin::Txid;
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::types::{Byte, Bytes, Result};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnIntIntTxInfos(pub Vec<BtcOnIntIntTxInfo>);

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct BtcOnIntIntTxInfo {
    pub host_token_amount: U256,
    pub user_data: Bytes,
    pub originating_tx_hash: Txid,
    pub int_token_address: String, // FIXME what if this becomes a string?
    pub originating_tx_address: String,
    pub destination_address: String,
    pub origin_chain_id: Bytes,
    pub destination_chain_id: Bytes,
    pub router_address: EthAddress,
    pub native_token_amount: u64,
}

impl BtcOnIntIntTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
