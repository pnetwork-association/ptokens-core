use std::str::FromStr;

use common::{errors::AppError, traits::DatabaseInterface, types::Result};
use serde::Deserialize;

use crate::{
    bitcoin_crate_alias::blockdata::{block::Block as BtcBlock, transaction::Transaction as BtcTransaction},
    btc_block::{BtcBlockAndId, BtcBlockJson},
    btc_utils::convert_hex_tx_to_btc_transaction,
    deposit_address_info::DepositAddressInfoJsonList,
    BtcState,
};

pub fn parse_btc_submission_json_and_put_in_state<'a, D: DatabaseInterface>(
    json_str: &str,
    state: BtcState<'a, D>,
) -> Result<BtcState<'a, D>> {
    info!("✔ Parsing BTC submission json and adding to state...");
    BtcSubmissionMaterialJson::from_str(json_str).and_then(|result| state.add_btc_submission_json(result))
}

pub fn parse_submission_material_and_put_in_state<'a, D: DatabaseInterface>(
    json_str: &str,
    state: BtcState<'a, D>,
) -> Result<BtcState<'a, D>> {
    info!("✔ Parsing BTC submisson material and adding to state...");
    BtcSubmissionMaterial::from_str(json_str).and_then(|result| state.add_btc_submission_material(result))
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Deserialize)]
pub struct BtcSubmissionMaterialJson {
    pub block: BtcBlockJson,
    pub any_sender: Option<bool>,
    pub transactions: Vec<String>,
    pub ref_block_num: Option<u16>,
    pub ref_block_prefix: Option<u32>,
    pub deposit_address_list: DepositAddressInfoJsonList,
}

impl FromStr for BtcSubmissionMaterialJson {
    type Err = AppError;

    fn from_str(string: &str) -> Result<Self> {
        info!("✔ Parsing `BtcSubmissionMaterialJson` from string...");
        match serde_json::from_str(string) {
            Ok(json) => Ok(json),
            Err(err) => Err(err.into()),
        }
    }
}

impl BtcSubmissionMaterialJson {
    fn convert_hex_txs_to_btc_transactions(hex_txs: Vec<String>) -> Result<Vec<BtcTransaction>> {
        hex_txs
            .into_iter()
            .map(convert_hex_tx_to_btc_transaction)
            .collect::<Result<Vec<BtcTransaction>>>()
    }

    pub fn to_btc_block(&self) -> Result<BtcBlock> {
        info!("✔ Parsing `BtcSubmissionMaterialJson` to `BtcBlock`...");
        Ok(BtcBlock {
            header: self.block.to_block_header()?,
            txdata: Self::convert_hex_txs_to_btc_transactions(self.transactions.clone())?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcSubmissionMaterial {
    pub ref_block_num: Option<u16>,
    pub block_and_id: BtcBlockAndId,
    pub ref_block_prefix: Option<u32>,
}

impl BtcSubmissionMaterial {
    pub fn from_json(json: &BtcSubmissionMaterialJson) -> Result<Self> {
        info!("✔ Parsing BTC submission material...");
        let submission_material = Self {
            ref_block_num: json.ref_block_num,
            ref_block_prefix: json.ref_block_prefix,
            block_and_id: BtcBlockAndId::from_json(json)?,
        };
        info!(
            "✔ BTC submission material parsed! Block number: {}",
            submission_material.block_and_id.height
        );
        Ok(submission_material)
    }
}

impl FromStr for BtcSubmissionMaterial {
    type Err = AppError;

    fn from_str(string: &str) -> Result<Self> {
        BtcSubmissionMaterialJson::from_str(string).and_then(|json| Self::from_json(&json))
    }
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use super::*;
    use crate::test_utils::{get_sample_btc_block_n, get_sample_btc_submission_material_json_string};

    #[test]
    fn should_get_submission_material_json_from_str() {
        let string = get_sample_btc_submission_material_json_string();
        let result = BtcSubmissionMaterialJson::from_str(&string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_submission_material_from_str() {
        let string = get_sample_btc_submission_material_json_string();
        let result = BtcSubmissionMaterial::from_str(&string);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_submission_material_correctly() {
        let result = get_sample_btc_block_n(13);
        assert!(result.is_ok());
    }
}
