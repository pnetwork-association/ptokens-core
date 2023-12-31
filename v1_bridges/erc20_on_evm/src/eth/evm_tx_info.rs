use std::str::FromStr;

use common::{
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};
use common_chain_ids::EthChainId;
use common_eth::{
    encode_erc777_mint_fxn_maybe_with_data,
    Erc20VaultPegInEventParams,
    EthDbUtilsExt,
    EthLog,
    EthLogExt,
    EthLogs,
    EthPrivateKey as EvmPrivateKey,
    EthReceipt,
    EthReceipts,
    EthState,
    EthSubmissionMaterial,
    EthTransaction as EvmTransaction,
    EthTransactions as EvmTransactions,
    ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
    MAX_BYTES_FOR_ETH_USER_DATA,
    ZERO_ETH_VALUE,
};
use common_metadata::{Metadata, MetadataAddress, MetadataChainId, MetadataProtocolId, ToMetadata};
use common_safe_addresses::safely_convert_str_to_eth_address;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::fees_calculator::{FeeCalculator, FeesCalculator};

#[cfg_attr(test, derive(Constructor))]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Erc20OnEvmEvmTxInfo {
    pub native_token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
    pub origin_chain_id: EthChainId,
}

impl ToMetadata for Erc20OnEvmEvmTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            // TODO Test for this case!
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new(
            &user_data,
            &MetadataAddress::new_from_eth_address(
                &self.token_sender,
                &MetadataChainId::from_str(&self.origin_chain_id.to_string())?,
            )?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}

impl FeeCalculator for Erc20OnEvmEvmTxInfo {
    fn get_amount(&self) -> U256 {
        debug!(
            "Getting token amount in `Erc20OnEvmEvmTxInfo` of {}",
            self.native_token_amount
        );
        self.native_token_amount
    }

    fn get_token_address(&self) -> EthAddress {
        debug!(
            "Getting token address in `Erc20OnEvmEvmTxInfo` of {}",
            self.eth_token_address
        );
        self.eth_token_address
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.native_token_amount {
            Err("Cannot subtract amount from `Erc20OnEvmEvmTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.native_token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `Erc20OnEvmEthTxInfo`!",
                subtrahend, self.native_token_amount, new_amount
            );
            Ok(self.update_amount(new_amount))
        }
    }
}

impl Erc20OnEvmEvmTxInfo {
    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.native_token_amount = new_amount;
        new_self
    }

    pub fn to_evm_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransaction> {
        info!("✔ Signing EVM transaction for tx info: {:?}", self);
        let operator_data = None;
        let metadata = if self.user_data.is_empty() {
            vec![]
        } else {
            self.to_metadata_bytes()?
        };
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        if !metadata.is_empty() {
            debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata))
        };
        encode_erc777_mint_fxn_maybe_with_data(
            &self.destination_address,
            &self.get_host_token_amount(dictionary)?,
            if metadata.is_empty() { None } else { Some(metadata) },
            operator_data,
        )
        .map(|data| {
            EvmTransaction::new_unsigned(
                data,
                nonce,
                ZERO_ETH_VALUE,
                self.evm_token_address,
                chain_id,
                gas_limit,
                gas_price,
            )
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(evm_private_key))
    }

    pub fn get_host_token_amount(&self, dictionary: &EthEvmTokenDictionary) -> Result<U256> {
        dictionary.convert_eth_amount_to_evm_amount(&self.eth_token_address, self.native_token_amount)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor, Deref, Serialize, Deserialize)]
pub struct Erc20OnEvmEvmTxInfos(pub Vec<Erc20OnEvmEvmTxInfo>);

impl FeesCalculator for Erc20OnEvmEvmTxInfos {
    fn get_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `Erc20OnEvmEvmTxInfo`...");
        self.iter()
            .map(|info| info.calculate_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if *fee == U256::zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<Erc20OnEvmEvmTxInfo>>>()?,
            ))
        })
    }
}

impl Erc20OnEvmEvmTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }

    fn get_host_token_amounts(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<U256>> {
        self.iter()
            .map(|tx_info| tx_info.get_host_token_amount(dictionary))
            .collect::<Result<Vec<U256>>>()
    }

    pub fn filter_out_zero_values(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        let host_token_amounts = self.get_host_token_amounts(dictionary)?;
        Ok(Self::new(
            self.iter()
                .zip(host_token_amounts.iter())
                .filter(|(tx_info, evm_amount)| match *evm_amount != &U256::zero() {
                    true => true,
                    false => {
                        info!(
                            "✘ Filtering out peg in info due to zero EVM asset amount: {:?}",
                            tx_info
                        );
                        false
                    },
                })
                .map(|(info, _)| info)
                .cloned()
                .collect::<Vec<Erc20OnEvmEvmTxInfo>>(),
        ))
    }

    fn is_log_erc20_on_evm_peg_in(log: &EthLog, vault_address: &EthAddress) -> Result<bool> {
        let log_contains_topic = log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC);
        let log_is_from_vault_address = log.address == *vault_address;
        Ok(log_contains_topic && log_is_from_vault_address)
    }

    fn receipt_contains_supported_erc20_on_evm_peg_in(receipt: &EthReceipt, vault_address: &EthAddress) -> bool {
        Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, vault_address).len() > 0
    }

    fn get_supported_erc20_on_evm_logs_from_receipt(receipt: &EthReceipt, vault_address: &EthAddress) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_erc20_on_evm_peg_in(log, vault_address), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Getting `ERC20-on-EVM` peg in infos from receipt...");
        Ok(Self::new(
            Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, vault_address)
                .iter()
                .map(|log| {
                    let event_params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let tx_info = Erc20OnEvmEvmTxInfo {
                        token_sender: event_params.token_sender,
                        origin_chain_id: origin_chain_id.clone(),
                        user_data: event_params.user_data.clone(),
                        eth_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        native_token_amount: event_params.token_amount,
                        destination_address: safely_convert_str_to_eth_address(&event_params.destination_address),
                        evm_token_address: dictionary.get_evm_address_from_eth_address(&event_params.token_address)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<Erc20OnEvmEvmTxInfo>>>()?,
        ))
    }

    fn filter_eth_submission_material_for_supported_peg_ins(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `ERC20-on-EVM` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    Erc20OnEvmEvmTxInfos::receipt_contains_supported_erc20_on_evm_peg_in(receipt, vault_address)
                })
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            None,
            None,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Getting `Erc20OnEvmEvmTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, vault_address, dictionary, origin_chain_id))
                .collect::<Result<Vec<Erc20OnEvmEvmTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<Erc20OnEvmEvmTxInfo>>>()
                .concat(),
        ))
    }

    pub fn to_evm_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `ERC20-on-EVM` EVM transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    Erc20OnEvmEvmTxInfo::to_evm_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_limit,
                        gas_price,
                        evm_private_key,
                        dictionary,
                    )
                })
                .collect::<Result<Vec<EvmTransaction>>>()?,
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `ERC20-on-EVM` peg-in infos...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|submission_material| {
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                Erc20OnEvmEvmTxInfos::from_submission_material(
                    &submission_material,
                    &state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                    &EthEvmTokenDictionary::get_from_db(state.db)?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                )
                .and_then(|infos| infos.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}

pub fn filter_out_zero_value_evm_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ NOT filtering out zero value `Erc20OnEvmEvmTxInfos` because there are none to filter!");
        Ok(state)
    } else {
        info!("✔ Maybe filtering out zero value `Erc20OnEvmEvmTxInfos`...");
        debug!(
            "✔ Num `Erc20OnEvmEvmTxInfos` before: {}",
            state.erc20_on_evm_evm_signed_txs.len()
        );
        Erc20OnEvmEvmTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| infos.filter_out_zero_values(&EthEvmTokenDictionary::get_from_db(state.db)?))
            .and_then(|infos| {
                debug!("✔ Num `Erc20OnEvmEvmTxInfos` after: {}", infos.len());
                infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `ERC20-on-EVM` peg in events...");
    let vault_address = state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?;
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(&vault_address, &[
            *ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
        ])
        .and_then(|filtered_submission_material| {
            Erc20OnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                &vault_address,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

pub fn maybe_sign_evm_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no EVM transactions to sign!");
        Ok(state)
    } else {
        let chain_id = state.evm_db_utils.get_eth_chain_id_from_db()?;
        Erc20OnEvmEvmTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                infos.to_evm_signed_txs(
                    state.evm_db_utils.get_eth_account_nonce_from_db()?,
                    &chain_id,
                    chain_id.get_erc777_mint_with_data_gas_limit(),
                    state.evm_db_utils.get_eth_gas_price_from_db()?,
                    &state.evm_db_utils.get_eth_private_key_from_db()?,
                    &EthEvmTokenDictionary::get_from_db(state.db)?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_erc20_on_evm_evm_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use common_eth::EthTxInfoCompatible;

    use super::*;
    use crate::test_utils::{
        get_eth_submission_material_n,
        get_sample_eth_evm_dictionary,
        get_sample_eth_evm_token_dictionary,
        get_sample_evm_private_key,
        get_sample_vault_address,
    };

    fn get_sample_tx_infos() -> Erc20OnEvmEvmTxInfos {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_eth_evm_token_dictionary();
        let origin_chain_id = EthChainId::Mainnet;
        Erc20OnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary, &origin_chain_id)
            .unwrap()
    }

    fn get_sample_tx_info() -> Erc20OnEvmEvmTxInfo {
        get_sample_tx_infos()[0].clone()
    }

    #[test]
    fn should_filter_submission_info_for_supported_redeems() {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let result =
            Erc20OnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(&material, &vault_address)
                .unwrap();
        let expected_num_receipts = 1;
        assert_eq!(result.receipts.len(), expected_num_receipts);
    }

    #[test]
    fn should_get_erc20_on_evm_evm_tx_info_from_submission_material() {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_eth_evm_token_dictionary();
        let origin_chain_id = EthChainId::Mainnet;
        let result =
            Erc20OnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary, &origin_chain_id)
                .unwrap();
        let expected_num_results = 1;
        assert_eq!(result.len(), expected_num_results);
        let expected_result = Erc20OnEvmEvmTxInfos::new(vec![Erc20OnEvmEvmTxInfo {
            user_data: vec![],
            native_token_amount: U256::from_dec_str("1000000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            eth_token_address: EthAddress::from_slice(
                &hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap(),
            ),
            // NOTE It's the `SAFE_EVM_ADDRESS_STR` ∵ @bertani accidentally included the `"`s in the pegin!
            destination_address: EthAddress::from_slice(
                &hex::decode("71a440ee9fa7f99fb9a697e96ec7839b8a1643b8").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("578670d0e08ca172eb8e862352e731814564fd6a12c3143e88bfb28292cd1535").unwrap(),
            ),
            origin_chain_id,
        }]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_signatures_from_evm_tx_info() {
        let dictionary = get_sample_eth_evm_dictionary();
        let pk = get_sample_evm_private_key();
        let infos = get_sample_tx_infos();
        let nonce = 0_u64;
        let chain_id = EthChainId::Rinkeby;
        let gas_limit = 300_000_usize;
        let gas_price = 20_000_000_000_u64;
        let signed_txs = infos
            .to_evm_signed_txs(nonce, &chain_id, gas_limit, gas_price, &pk, &dictionary)
            .unwrap();
        let expected_num_results = 1;
        assert_eq!(signed_txs.len(), expected_num_results);
        let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
        let expected_tx_hex = "f8aa808504a817c800830493e094daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af9280b84440c10f1900000000000000000000000071a440ee9fa7f99fb9a697e96ec7839b8a1643b80000000000000000000000000000000000000000000000000de0b6b3a76400002ca086b9b9a4de05a798e0af067ee3feff7008057c1feeab8f42db5710bd69b949eba0016e1e143d02596a21a0fb10202a9343a279d5862e1bf300d6af57e65887fc7e"
;
        assert_eq!(tx_hex, expected_tx_hex);
    }

    #[test]
    fn should_calculate_eth_on_evm_evm_tx_info_fee() {
        let info = get_sample_tx_info();
        let fee_basis_points = 25;
        let result = info.calculate_fee(fee_basis_points);
        let expected_result = U256::from_dec_str("2500000000000000").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_amount_from_eth_on_evm_evm_tx_info() {
        let info = get_sample_tx_info();
        let subtrahend = U256::from(1337);
        let result = info.subtract_amount(subtrahend).unwrap();
        let expected_native_token_amount = U256::from_dec_str("999999999999998663").unwrap();
        assert_eq!(result.native_token_amount, expected_native_token_amount)
    }

    #[test]
    fn should_fail_to_subtract_too_large_amount_from_eth_on_evm_evm_tx_info() {
        let info = get_sample_tx_info();
        let subtrahend = info.native_token_amount + 1;
        let result = info.subtract_amount(subtrahend);
        assert!(result.is_err());
    }
}
