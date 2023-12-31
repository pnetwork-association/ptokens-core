use std::str::FromStr;

use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_chain_ids::EosChainId;
use common_eos::{
    get_eos_tx_expiration_timestamp_with_offset,
    get_signed_eos_ptoken_issue_tx,
    get_symbol_from_eos_asset,
    EosDbUtils,
    EosPrivateKey,
    EosSignedTransaction,
    EosSignedTransactions,
};
use common_eth::EthState;
use common_metadata::MetadataChainId;

use crate::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos};

impl IntOnEosEosTxInfos {
    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        private_key: &EosPrivateKey,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `erc20-on-eos` peg in infos...", self.len());
        Ok(EosSignedTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, info)| {
                    info.to_eos_signed_tx(
                        ref_block_num,
                        ref_block_prefix,
                        chain_id,
                        private_key,
                        get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                        dictionary,
                    )
                })
                .collect::<Result<Vec<EosSignedTransaction>>>()?,
        ))
    }
}

impl IntOnEosEosTxInfo {
    pub fn to_eos_signed_tx(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        private_key: &EosPrivateKey,
        timestamp: u32,
        dictionary: &EosEthTokenDictionary,
    ) -> Result<EosSignedTransaction> {
        info!("✔ Signing EOS tx from `IntOnEosEosTxInfo`: {:?}", self);
        let dictionary_entry = dictionary.get_entry_via_eos_address_and_symbol(
            get_symbol_from_eos_asset(&self.eos_asset_amount),
            &self.eos_token_address,
        )?;
        let eos_amount = dictionary_entry.convert_u256_to_eos_asset_string(&self.token_amount)?;
        get_signed_eos_ptoken_issue_tx(
            ref_block_num,
            ref_block_prefix,
            &self.destination_address,
            &eos_amount,
            chain_id,
            private_key,
            &self.eos_token_address,
            timestamp,
            if self.user_data.is_empty() {
                None
            } else {
                info!("✔ Wrapping `user_data` in metadata for `IntOnEosEosTxInfo¬");
                Some(
                    self.to_metadata()?
                        .to_bytes_for_protocol(&MetadataChainId::from_str(&chain_id.to_string())?.to_protocol_id())?,
                )
            },
        )
    }
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ NOT signing `INT-on-EOS` EOS txs because there's none to sign!");
        Ok(state)
    } else {
        info!("✔ Maybe signing `INT-on-EOS` EOS txs...");
        let submission_material = state.get_eth_submission_material()?;
        IntOnEosEosTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                tx_infos.to_eos_signed_txs(
                    submission_material.get_eos_ref_block_num()?,
                    submission_material.get_eos_ref_block_prefix()?,
                    &EosDbUtils::new(state.db).get_eos_chain_id_from_db()?,
                    &EosPrivateKey::get_from_db(state.db)?,
                    &EosEthTokenDictionary::get_from_db(state.db)?,
                )
            })
            .and_then(|signed_txs| signed_txs.to_bytes())
            .and_then(|bytes| state.add_signed_txs(bytes))
    }
}
