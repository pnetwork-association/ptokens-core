use std::str::FromStr;

use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Action as EosAction, PermissionLevel, Transaction as EosTransaction};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::{
        eos::{
            eos_actions::PTokenPegOutAction,
            eos_chain_id::EosChainId,
            eos_constants::EOS_ACCOUNT_PERMISSION_LEVEL,
            eos_crypto::{
                eos_private_key::EosPrivateKey,
                eos_transaction::{EosSignedTransaction, EosSignedTransactions},
            },
            eos_database_utils::{get_eos_account_name_from_db, get_eos_chain_id_from_db},
            eos_utils::{
                get_eos_tx_expiration_timestamp_with_offset,
                parse_eos_account_name_or_default_to_safe_address,
            },
        },
        eth::{
            eth_contracts::erc777::{
                Erc777RedeemEvent,
                ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
            },
            eth_database_utils::get_eth_canon_block_from_db,
            eth_log::EthLog,
            eth_state::EthState,
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    eos_on_eth::{
        constants::MINIMUM_WEI_AMOUNT,
        fees_calculator::{FeeCalculator, FeesCalculator},
    },
    traits::DatabaseInterface,
    types::{Byte, Result},
};

const ZERO_ETH_ASSET_STR: &str = "0.0000 EOS";

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EosOnEthEthTxInfos(pub Vec<EosOnEthEthTxInfo>);

impl EosOnEthEthTxInfos {
    pub fn from_eth_submission_material(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
    ) -> Result<Self> {
        Self::from_eth_submission_material_without_filtering(material, token_dictionary).map(|tx_infos| {
            debug!("Num tx infos before filtering: {}", tx_infos.len());
            let filtered = tx_infos.filter_out_those_with_zero_eos_asset_amount(token_dictionary);
            debug!("Num tx infos after filtering: {}", filtered.len());
            filtered
        })
    }

    fn from_eth_submission_material_without_filtering(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
    ) -> Result<Self> {
        let eth_contract_addresses = token_dictionary.to_eth_addresses();
        debug!("Addresses from dict: {:?}", eth_contract_addresses);
        Ok(Self(
            material
                .receipts
                .get_receipts_containing_log_from_addresses_and_with_topics(&eth_contract_addresses, &[
                    *ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                    *ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
                ])
                .iter()
                .map(|receipt| {
                    receipt
                        .get_logs_from_addresses_with_topics(&eth_contract_addresses, &[
                            *ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                            *ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
                        ])
                        .iter()
                        .map(|log| EosOnEthEthTxInfo::from_eth_log(log, &receipt.transaction_hash, token_dictionary))
                        .collect::<Result<Vec<EosOnEthEthTxInfo>>>()
                })
                .collect::<Result<Vec<Vec<EosOnEthEthTxInfo>>>>()?
                .concat(),
        ))
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnEthEthTxInfos::new(
            self.iter()
                .filter(|info| {
                    if info.token_amount >= min_amount {
                        true
                    } else {
                        info!("✘ Filtering out tx info ∵ value too low: {:?}", info);
                        false
                    }
                })
                .cloned()
                .collect::<Vec<EosOnEthEthTxInfo>>(),
        ))
    }

    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        pk: &EosPrivateKey,
        eos_smart_contract: &EosAccountName,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `EosOnEthEthTxInfos`...", self.len());
        Ok(EosSignedTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    info!("✔ Signing EOS tx from `EosOnEthEthTxInfo`: {:?}", tx_info);
                    tx_info.to_eos_signed_tx(
                        ref_block_num,
                        ref_block_prefix,
                        eos_smart_contract,
                        ZERO_ETH_ASSET_STR,
                        chain_id,
                        pk,
                        get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                    )
                })
                .collect::<Result<Vec<EosSignedTransaction>>>()?,
        ))
    }

    fn filter_out_those_with_zero_eos_asset_amount(&self, dictionary: &EosEthTokenDictionary) -> Self {
        info!("✔ Filtering out `EosOnEthEthTxInfos` if they have a zero EOS asset amount...");
        Self::new(
            self.iter()
                .filter(|tx_info| {
                    match dictionary.get_zero_eos_asset_amount_via_eth_token_address(&tx_info.eth_token_address) {
                        Err(_) => {
                            info!(
                                "✘ Filtering out tx ∵ cannot determine zero EOS asset amount! {:?}",
                                tx_info
                            );
                            false
                        },
                        Ok(zero_asset_amount) => tx_info.eos_asset_amount != zero_asset_amount,
                    }
                })
                .cloned()
                .collect::<Vec<EosOnEthEthTxInfo>>(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct EosOnEthEthTxInfo {
    pub token_amount: U256,
    pub eos_address: String,
    pub eos_token_address: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub eth_token_address: EthAddress,
    pub originating_tx_hash: EthHash,
}

impl FeeCalculator for EosOnEthEthTxInfo {
    fn get_amount(&self) -> U256 {
        info!("✔Getting token amount in `EosOnEthEthTxInfo` of {}", self.token_amount);
        self.token_amount
    }

    fn get_eth_token_address(&self) -> EthAddress {
        debug!(
            "Getting EOS token address in `EosOnEthEthTxInfo` of {}",
            self.eth_token_address
        );
        self.eth_token_address
    }

    fn get_eos_token_address(&self) -> Result<EosAccountName> {
        debug!(
            "Getting EOS token address in `EosOnEthEthTxInfo` of {}",
            self.eos_token_address
        );
        Ok(EosAccountName::from_str(&self.eos_token_address)?)
    }

    // FIXME: Can I not impl this on the trait itself?
    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.token_amount {
            Err("Cannot subtract amount from `EosOnEthEthTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `EosOnEthEthTxInfo`!",
                subtrahend, self.token_amount, new_amount
            );
            let mut new_self = self.clone();
            new_self.token_amount = new_amount;
            Ok(new_self)
        }
    }
}

impl EosOnEthEthTxInfo {
    pub fn from_eth_log(log: &EthLog, tx_hash: &EthHash, token_dictionary: &EosEthTokenDictionary) -> Result<Self> {
        info!("✔ Parsing `EosOnEthEthTxInfo` from ETH log...");
        Erc777RedeemEvent::from_eth_log(log).and_then(|params| {
            Ok(Self {
                token_amount: params.value,
                originating_tx_hash: *tx_hash,
                token_sender: params.redeemer,
                eth_token_address: log.address,
                eos_token_address: token_dictionary.get_eos_account_name_from_eth_token_address(&log.address)?,
                eos_asset_amount: token_dictionary.convert_u256_to_eos_asset_string(&log.address, &params.value)?,
                eos_address: parse_eos_account_name_or_default_to_safe_address(&params.underlying_asset_recipient)?
                    .to_string(),
            })
        })
    }

    fn get_eos_ptoken_peg_out_action(
        from: &str,
        actor: &str,
        permission_level: &str,
        token_contract: &str,
        quantity: &str,
        recipient: &str,
        metadata: &[Byte],
    ) -> Result<EosAction> {
        debug!(
            "from: {}\nactor: {}\npermission_level: {}\ntoken_contract: {}\nquantity: {}\nrecipient: {}\nmetadata: '0x{}'",
            from, actor, permission_level, token_contract, quantity, recipient, hex::encode(metadata),
        );
        Ok(EosAction::from_str(
            from,
            "pegout",
            vec![PermissionLevel::from_str(actor, permission_level)?],
            PTokenPegOutAction::from_str(token_contract, quantity, recipient, metadata)?,
        )?)
    }

    pub fn to_eos_signed_tx(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        eos_smart_contract: &EosAccountName,
        amount: &str,
        chain_id: &EosChainId,
        pk: &EosPrivateKey,
        timestamp: u32,
    ) -> Result<EosSignedTransaction> {
        info!("✔ Signing eos tx...");
        debug!(
            "smart-contract: {}\namount: {}\nchain ID: {}",
            &eos_smart_contract,
            &amount,
            &chain_id.to_hex()
        );
        Self::get_eos_ptoken_peg_out_action(
            &eos_smart_contract.to_string(),
            &eos_smart_contract.to_string(),
            EOS_ACCOUNT_PERMISSION_LEVEL,
            &self.eos_token_address,
            &self.eos_asset_amount,
            &self.eos_address,
            &[], // NOTE: Empty metadata for now.
        )
        .map(|action| EosTransaction::new(timestamp, ref_block_num, ref_block_prefix, vec![action]))
        .and_then(|ref unsigned_tx| {
            EosSignedTransaction::from_unsigned_tx(&eos_smart_contract.to_string(), amount, chain_id, pk, unsigned_tx)
        })
    }
}

pub fn maybe_parse_eth_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `eos-on-eth` tx infos...");
    get_eth_canon_block_from_db(&state.db).and_then(|material| match material.receipts.is_empty() {
        true => {
            info!("✔ No receipts in canon block ∴ no info to parse!");
            Ok(state)
        },
        false => {
            info!(
                "✔ {} receipts in canon block ∴ parsing ETH tx info...",
                material.receipts.len()
            );
            EosOnEthEthTxInfos::from_eth_submission_material(&material, state.get_eos_eth_token_dictionary()?)
                .and_then(|tx_infos| state.add_eos_on_eth_eth_tx_infos(tx_infos))
        },
    })
}

pub fn maybe_filter_out_eth_tx_info_with_value_too_low_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering `EosOnEthEthTxInfos`...");
    debug!("✔ Num tx infos before: {}", state.eos_on_eth_eth_tx_infos.len());
    state
        .eos_on_eth_eth_tx_infos
        .filter_out_those_with_value_too_low()
        .and_then(|filtered_infos| {
            debug!("✔ Num tx infos after: {}", filtered_infos.len());
            state.replace_eos_on_eth_eth_tx_infos(filtered_infos)
        })
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe signing `EosOnEthEthTxInfos`...");
    let submission_material = state.get_eth_submission_material()?;
    state
        .eos_on_eth_eth_tx_infos
        .to_eos_signed_txs(
            submission_material.get_eos_ref_block_num()?,
            submission_material.get_eos_ref_block_prefix()?,
            &get_eos_chain_id_from_db(&state.db)?,
            &EosPrivateKey::get_from_db(&state.db)?,
            &get_eos_account_name_from_db(&state.db)?,
        )
        .and_then(|signed_txs| state.add_eos_transactions(signed_txs))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        dictionaries::eos_eth::EosEthTokenDictionaryEntry,
        eos_on_eth::test_utils::{
            get_eth_submission_material_n,
            get_eth_submission_material_with_bad_eos_account_name,
            get_eth_submission_material_with_two_peg_ins,
            get_sample_eos_eth_token_dictionary,
        },
    };

    fn get_sample_eos_private_key() -> EosPrivateKey {
        EosPrivateKey::from_slice(
            &hex::decode("17b116e5e55af3b9985ff6c6e0320578176b83ca55570a66683d3b36d9deca64").unwrap(),
        )
        .unwrap()
    }

    fn get_sample_eos_on_eth_eth_tx_infos() -> EosOnEthEthTxInfos {
        EosOnEthEthTxInfos::from_eth_submission_material(
            &get_eth_submission_material_n(1).unwrap(),
            &get_sample_eos_eth_token_dictionary(),
        )
        .unwrap()
    }

    fn get_sample_eos_on_eth_eth_tx_info() -> EosOnEthEthTxInfo {
        get_sample_eos_on_eth_eth_tx_infos()[0].clone()
    }

    #[test]
    fn should_get_tx_info_from_eth_submission_material() {
        let tx_infos = get_sample_eos_on_eth_eth_tx_infos();
        let result = tx_infos[0].clone();
        let expected_token_amount = U256::from_dec_str("100000000000000").unwrap();
        let expected_eos_address = "whateverxxxx";
        let expected_eos_token_address = "eosio.token".to_string();
        let expected_eos_asset_amount = "0.0001 EOS".to_string();
        let expected_token_sender =
            EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap());
        let expected_eth_token_address =
            EthAddress::from_slice(&hex::decode("711c50b31ee0b9e8ed4d434819ac20b4fbbb5532").unwrap());
        let expected_originating_tx_hash = EthHash::from_slice(
            &hex::decode("9b9b2b88bdd495c132704154003d2deb65bd34ce6f8836ed6efdf0ba9def2b3e").unwrap(),
        );
        assert_eq!(result.token_amount, expected_token_amount);
        assert_eq!(result.eos_address, expected_eos_address);
        assert_eq!(result.eos_token_address, expected_eos_token_address);
        assert_eq!(result.eos_asset_amount, expected_eos_asset_amount);
        assert_eq!(result.token_sender, expected_token_sender);
        assert_eq!(result.eth_token_address, expected_eth_token_address);
        assert_eq!(result.originating_tx_hash, expected_originating_tx_hash);
    }

    #[test]
    fn should_get_eos_signed_txs_from_tx_info() {
        let tx_infos = get_sample_eos_on_eth_eth_tx_infos();
        let ref_block_num = 1;
        let ref_block_prefix = 1;
        let chain_id = EosChainId::EosMainnet;
        let pk = get_sample_eos_private_key();
        let eos_smart_contract = EosAccountName::from_str("11ppntoneos").unwrap();
        let result = tx_infos
            .to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &eos_smart_contract)
            .unwrap()[0]
            .transaction
            .clone();
        let expected_result = "010001000000000000000100305593e6596b0800000000644d99aa0100305593e6596b0800000000a8ed32322100a6823403ea3055010000000000000004454f5300000000d07bef576d954de30000";
        let result_with_no_timestamp = &result[8..];
        assert_eq!(result_with_no_timestamp, expected_result);
    }

    #[test]
    fn should_filter_out_zero_eth_amounts() {
        let dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(
            "{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TOK\",\"eos_symbol\":\"EOS\",\"eth_address\":\"9a74c1e17b31745199b229b5c05b61081465b329\",\"eos_address\":\"eosio.token\"}"
        ).unwrap()]);
        let submission_material = get_eth_submission_material_n(2).unwrap();
        let expected_result_before = 1;
        let expected_result_after = 0;
        let result_before =
            EosOnEthEthTxInfos::from_eth_submission_material_without_filtering(&submission_material, &dictionary)
                .unwrap();
        assert_eq!(result_before.len(), expected_result_before);
        assert_eq!(result_before[0].eos_asset_amount, "0.0000 EOS");
        let result_after = result_before.filter_out_those_with_zero_eos_asset_amount(&dictionary);
        assert_eq!(result_after.len(), expected_result_after);
    }

    #[test]
    fn should_default_to_safe_address_when_signing_tx_with_bad_eos_account_name_in_submission_material() {
        let token_dictionary_entry_str = "{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TLOS\",\"eos_symbol\":\"TLOS\",\"eth_address\":\"b6c53431608e626ac81a9776ac3e999c5556717c\",\"eos_address\":\"eosio.token\"}";
        let token_dictionary =
            EosEthTokenDictionary::new(vec![
                EosEthTokenDictionaryEntry::from_str(&token_dictionary_entry_str).unwrap()
            ]);
        let submission_material = get_eth_submission_material_with_bad_eos_account_name();
        let tx_infos =
            EosOnEthEthTxInfos::from_eth_submission_material(&submission_material, &token_dictionary).unwrap();
        let ref_block_num = 1;
        let ref_block_prefix = 2;
        let chain_id = EosChainId::EosMainnet;
        let eos_smart_contract = EosAccountName::from_str("11ppntoneos").unwrap();
        let pk = get_sample_eos_private_key();
        let result = tx_infos.to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &eos_smart_contract);
        assert!(result.is_ok());
    }

    #[test]
    fn same_param_tx_infos_should_not_create_same_signatures() {
        let submission_material = get_eth_submission_material_with_two_peg_ins();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let tx_infos = EosOnEthEthTxInfos::from_eth_submission_material(&submission_material, &dictionary).unwrap();
        let ref_block_num = 1;
        let ref_block_prefix = 2;
        let chain_id = EosChainId::EosMainnet;
        let eos_smart_contract = EosAccountName::from_str("11ppntoneos").unwrap();
        let pk = get_sample_eos_private_key();
        let result = tx_infos
            .to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &eos_smart_contract)
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_ne!(result[0], result[1]);
    }

    #[test]
    fn should_subtract_amount_from_eos_on_eth_eth_tx_info() {
        let info = get_sample_eos_on_eth_eth_tx_info();
        let subtrahend = U256::from(1337);
        let mut expected_result = info.clone();
        expected_result.token_amount = U256::from_dec_str("99999999998663").unwrap();
        let result = info.subtract_amount(subtrahend).unwrap();
        assert_eq!(result, expected_result);
    }
}
