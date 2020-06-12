use ethereum_types::{
    U256,
    Address as EthAddress
};
use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    types::{
        Bytes,
        Result,
    },
    chains::btc::deposit_address_info::{
        DepositAddressInfo,
        DepositAddressInfoJson,
    },
    btc_on_eth::{
        constants::SAFE_BTC_ADDRESS,
        utils::convert_hex_to_address,
    },
};
use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
    blockdata::{
        block::Block as BtcBlock,
        transaction::Transaction as BtcTransaction,
    },
};

pub type BtcTransactions = Vec<BtcTransaction>;
pub type MintingParams = Vec<MintingParamStruct>;
pub type DepositInfoList = Vec<DepositAddressInfo>;
pub type BtcRecipientsAndAmounts = Vec<BtcRecipientAndAmount>;
pub type DepositAddressJsonList = Vec<DepositAddressInfoJson>;
pub type DepositInfoHashMap =  HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcRecipientAndAmount {
    pub amount: u64,
    pub recipient: BtcAddress,
}

impl BtcRecipientAndAmount {
    pub fn new(recipient: &str, amount: u64) -> Result<Self> {
        Ok(
            BtcRecipientAndAmount {
                amount,
                recipient: match BtcAddress::from_str(recipient) {
                    Ok(address) => address,
                    Err(error) => {
                        info!(
                            "✔ Error parsing BTC address for recipient: {}",
                            error
                        );
                        info!(
                            "✔ Defaulting to SAFE BTC address: {}",
                            SAFE_BTC_ADDRESS,
                        );
                        BtcAddress::from_str(SAFE_BTC_ADDRESS)?
                    }
                }
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockInDbFormat {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub extra_data: Bytes,
    pub minting_params: MintingParams,
}

impl BtcBlockInDbFormat {
    pub fn new(
        height: u64,
        id: sha256d::Hash,
        minting_params: MintingParams,
        block: BtcBlock,
        extra_data: Bytes,
    ) -> Result<Self> {
        Ok(BtcBlockInDbFormat{ id, block, height, minting_params, extra_data })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintingParamStruct {
    pub amount: U256,
    pub eth_address: EthAddress,
    pub originating_tx_hash: sha256d::Hash,
    pub originating_tx_address: String,
}

impl MintingParamStruct {
    pub fn new(
        amount: U256,
        eth_address: String,
        originating_tx_hash: sha256d::Hash,
        originating_tx_address: BtcAddress,
    ) -> Result<MintingParamStruct> {
        Ok(
            MintingParamStruct {
                amount,
                originating_tx_hash,
                eth_address: convert_hex_to_address(eth_address)?,
                originating_tx_address: originating_tx_address.to_string(),
            }
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BtcBlockAndTxsJson {
    pub block: BtcBlockJson,
    pub transactions: Vec<String>,
    pub deposit_address_list: DepositAddressJsonList,

    #[cfg(feature = "any-sender")]
    pub any_sender: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BtcBlockJson {
    pub bits: u32,
    pub id: String,
    pub nonce: u32,
    pub version: u32,
    pub height: u64,
    pub timestamp: u32,
    pub merkle_root: String,
    pub previousblockhash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockAndId {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub deposit_address_list: DepositInfoList,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}
