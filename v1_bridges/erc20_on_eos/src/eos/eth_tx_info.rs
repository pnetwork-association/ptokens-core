use std::str::{from_utf8, FromStr};

use common::{
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{convert_bytes_to_u64, strip_hex_prefix},
};
use common_chain_ids::EosChainId;
use common_eos::{EosActionProof, EosState, GlobalSequence, GlobalSequences, ProcessedGlobalSequences};
use common_eth::{EthDbUtils, EthDbUtilsExt, MAX_BYTES_FOR_ETH_USER_DATA};
use common_metadata::{Metadata, MetadataAddress, MetadataChainId, MetadataProtocolId, ToMetadata};
use common_safe_addresses::SAFE_ETH_ADDRESS;
use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::fees_calculator::{FeeCalculator, FeesCalculator};

#[derive(Clone, Debug, PartialEq, Eq, Default, Deref, Constructor, Serialize, Deserialize)]
pub struct Erc20OnEosEthTxInfos(pub Vec<Erc20OnEosEthTxInfo>);

impl Erc20OnEosEthTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        if self.is_empty() {
            Ok(vec![])
        } else {
            Ok(serde_json::to_vec(&self)?)
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }
}

#[cfg_attr(test, derive(Constructor))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Erc20OnEosEthTxInfo {
    pub amount: U256,
    pub from: EosAccountName,
    pub destination_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub eos_token_address: String,
    pub eos_tx_amount: String,
    pub user_data: Bytes,
    pub origin_chain_id: EosChainId,
    pub eth_vault_address: EthAddress,
}

impl FeesCalculator for Erc20OnEosEthTxInfos {
    fn get_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `Erc20OnEosEthTxInfos`...");
        self.iter()
            .map(|info| info.calculate_peg_out_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if fee.is_zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<Erc20OnEosEthTxInfo>>>()?,
            ))
        })
    }
}

impl Erc20OnEosEthTxInfos {
    pub fn get_global_sequences(&self) -> GlobalSequences {
        GlobalSequences::new(
            self.iter()
                .map(|infos| infos.global_sequence)
                .collect::<Vec<GlobalSequence>>(),
        )
    }

    pub fn from_action_proofs(
        action_proofs: &[EosActionProof],
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EosChainId,
        eth_vault_address: &EthAddress,
    ) -> Result<Erc20OnEosEthTxInfos> {
        Ok(Erc20OnEosEthTxInfos::new(
            action_proofs
                .iter()
                .map(|action_proof| {
                    Erc20OnEosEthTxInfo::from_action_proof(action_proof, dictionary, origin_chain_id, eth_vault_address)
                })
                .collect::<Result<Vec<Erc20OnEosEthTxInfo>>>()?,
        ))
    }

    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedGlobalSequences) -> Result<Self> {
        Ok(Erc20OnEosEthTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<Erc20OnEosEthTxInfo>>(),
        ))
    }
}

impl FeeCalculator for Erc20OnEosEthTxInfo {
    fn get_amount(&self) -> U256 {
        info!("✔ Getting token amount in `Erc20OnEosEthTxInfo` of {}", self.amount);
        self.amount
    }

    fn get_eth_token_address(&self) -> EthAddress {
        debug!(
            "Getting EOS token address in `Erc20OnEvmEvmTxInfo` of {}",
            self.eth_token_address
        );
        self.eth_token_address
    }

    fn get_eos_token_address(&self) -> Result<EosAccountName> {
        debug!(
            "Getting EOS token address in `Erc20OnEvmEvmTxInfo` of {}",
            self.eos_token_address
        );
        Ok(EosAccountName::from_str(&self.eos_token_address)?)
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.amount {
            Err("Cannot subtract amount from `Erc20OnEosEthTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `Erc20OnEosEthTxInfo`!",
                subtrahend, self.amount, new_amount
            );
            let mut new_self = self.clone();
            new_self.amount = new_amount;
            Ok(new_self)
        }
    }
}

impl ToMetadata for Erc20OnEosEthTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes!",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new(
            &user_data,
            &MetadataAddress::new_from_eos_address(
                &self.from,
                &MetadataChainId::from_str(&self.origin_chain_id.to_string())?,
            )?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}

impl Erc20OnEosEthTxInfo {
    fn get_memo_string_from_proof(proof: &EosActionProof) -> Result<String> {
        proof
            .check_proof_action_data_length(25, "Not enough data to parse `Erc20OnEosEthTxInfo` memo from proof!")
            .and_then(|_| Ok(from_utf8(&proof.action.data[25..])?.to_string()))
    }

    fn get_erc20_on_eos_eth_redeem_address(proof: &EosActionProof) -> Result<EthAddress> {
        Ok(EthAddress::from_slice(&hex::decode(strip_hex_prefix(
            &Self::get_memo_string_from_proof(proof)?,
        ))?))
    }

    fn get_redeem_address_from_proof_or_default_to_safe_address(proof: &EosActionProof) -> Result<EthAddress> {
        match Self::get_erc20_on_eos_eth_redeem_address(proof) {
            Ok(address) => Ok(address),
            Err(_) => {
                info!(
                    "✘ Could not parse ETH address from action memo: {}",
                    Self::get_memo_string_from_proof(proof)?
                );
                info!("✔ Defaulting to safe ETH address: 0x{}", hex::encode(*SAFE_ETH_ADDRESS));
                Ok(*SAFE_ETH_ADDRESS)
            },
        }
    }

    fn get_redeem_amount_from_proof(
        proof: &EosActionProof,
        dictionary_entry: &EosEthTokenDictionaryEntry,
    ) -> Result<U256> {
        proof
            .check_proof_action_data_length(15, "Not enough data to parse `Erc20OnEosEthTxInfo` amount from proof!")
            .and_then(|_| {
                Ok(dictionary_entry.convert_u64_to_eos_asset(convert_bytes_to_u64(&proof.action.data[8..=15])?))
            })
            .and_then(|eos_asset| dictionary_entry.convert_eos_asset_to_eth_amount(&eos_asset))
    }

    pub fn from_action_proof(
        proof: &EosActionProof,
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EosChainId,
        eth_vault_address: &EthAddress,
    ) -> Result<Self> {
        dictionary
            .get_entry_via_eos_address(&proof.get_action_eos_account())
            .and_then(|entry| {
                let amount = Self::get_redeem_amount_from_proof(proof, &entry)?;
                let eos_tx_amount = entry.convert_u256_to_eos_asset_string(&amount)?;
                info!("✔ Converting action proof to `erc20-on-eos` redeem info...");
                Ok(Self {
                    amount,
                    eos_tx_amount,
                    originating_tx_id: proof.tx_id,
                    eth_token_address: entry.eth_address,
                    from: proof.get_action_sender()?,
                    eos_token_address: entry.eos_address,
                    global_sequence: proof.action_receipt.global_sequence,
                    destination_address: Self::get_redeem_address_from_proof_or_default_to_safe_address(proof)?,
                    user_data: vec![], // NOTE: proof.get_user_data() currently unimplemented!,
                    origin_chain_id: origin_chain_id.clone(),
                    eth_vault_address: *eth_vault_address,
                })
            })
    }
}

pub fn maybe_parse_eth_tx_infos_and_put_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Parsing redeem params from actions data...");
    Erc20OnEosEthTxInfos::from_action_proofs(
        &state.action_proofs,
        state.get_eos_eth_token_dictionary()?,
        &state.eos_db_utils.get_eos_chain_id_from_db()?,
        &EthDbUtils::new(state.db).get_erc20_on_eos_smart_contract_address_from_db()?,
    )
    .and_then(|eth_tx_infos| {
        info!("✔ Parsed {} redeem infos!", eth_tx_infos.len());
        let global_seqs = eth_tx_infos.get_global_sequences();
        Ok(state
            .add_global_sequences(global_seqs)
            .add_tx_infos(eth_tx_infos.to_bytes()?))
    })
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ NOT filtering out already processed tx IDs because there are none to filter!");
        Ok(state)
    } else {
        info!("✔ Filtering out already processed tx IDs...");
        Erc20OnEosEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| infos.filter_out_already_processed_txs(&state.processed_tx_ids))
            .and_then(|filtered| filtered.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::{dictionaries::eos_eth::test_utils::get_sample_eos_eth_token_dictionary, errors::AppError};
    use common_eos::convert_hex_to_checksum256;

    use super::*;
    use crate::test_utils::get_sample_eos_submission_material_n;

    fn get_sample_action_proof_for_erc20_redeem() -> EosActionProof {
        get_sample_eos_submission_material_n(10).action_proofs[0].clone()
    }

    fn get_sample_erc20_on_eos_eth_tx_info() -> Erc20OnEosEthTxInfo {
        let user_data = vec![];
        let origin_chain_id = EosChainId::EosMainnet;
        let eos_account_name = "testpethxxxx".to_string();
        Erc20OnEosEthTxInfo::new(
            U256::from_dec_str("1337000000000").unwrap(),
            EosAccountName::from_str("t11ptokens11").unwrap(),
            EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap()),
            EthAddress::from_slice(&hex::decode("32eF9e9a622736399DB5Ee78A68B258dadBB4353").unwrap()),
            convert_hex_to_checksum256("ed991197c5d571f39b4605f91bf1374dd69237070d44b46d4550527c245a01b9").unwrap(),
            250255005734,
            eos_account_name,
            "0.000001337 PETH".to_string(),
            user_data,
            origin_chain_id,
            EthAddress::default(),
        )
    }

    fn get_sample_erc20_on_eos_eth_tx_infos() -> Erc20OnEosEthTxInfos {
        Erc20OnEosEthTxInfos::new(vec![
            get_sample_erc20_on_eos_eth_tx_info(),
            get_sample_erc20_on_eos_eth_tx_info(),
        ])
    }

    #[test]
    fn should_serde_empty_eth_tx_info_correctly() {
        let info = Erc20OnEosEthTxInfos::default();
        let result = info.to_bytes().unwrap();
        let expected_result: Bytes = vec![];
        assert_eq!(result, expected_result);
        let result_2 = Erc20OnEosEthTxInfos::from_bytes(&result).unwrap();
        assert_eq!(result_2, info);
    }

    #[test]
    fn should_get_erc20_on_eos_eth_redeem_amount() {
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let dictionary_entry = EosEthTokenDictionaryEntry::new(
            18,
            9,
            "PETH".to_string(),
            "SAM".to_string(),
            "testpethxxxx".to_string(),
            EthAddress::from_slice(&hex::decode("32eF9e9a622736399DB5Ee78A68B258dadBB4353").unwrap()),
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        );
        let proof = get_sample_action_proof_for_erc20_redeem();
        let result = Erc20OnEosEthTxInfo::get_redeem_amount_from_proof(&proof, &dictionary_entry).unwrap();
        let expected_result = U256::from_dec_str("1337000000000").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_erc20_on_eos_eth_redeem_address() {
        let expected_result = EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap());
        let proof = get_sample_action_proof_for_erc20_redeem();
        let result = Erc20OnEosEthTxInfo::get_redeem_address_from_proof_or_default_to_safe_address(&proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_proof_to_erc20_on_eos_eth_tx_info() {
        let eos_account_name = "testpethxxxx".to_string();
        let expected_result = get_sample_erc20_on_eos_eth_tx_info();
        let origin_chain_id = EosChainId::EosMainnet;
        let eth_basis_points = 0;
        let eos_basis_points = 0;
        let dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::new(
            18,
            9,
            "PETH".to_string(),
            "SAM".to_string(),
            eos_account_name,
            EthAddress::from_slice(&hex::decode("32eF9e9a622736399DB5Ee78A68B258dadBB4353").unwrap()),
            eth_basis_points,
            eos_basis_points,
            U256::zero(),
            0,
            0,
            "".to_string(),
        )]);
        let proof = get_sample_action_proof_for_erc20_redeem();
        let vault_address = EthAddress::default();
        let result =
            Erc20OnEosEthTxInfo::from_action_proof(&proof, &dictionary, &origin_chain_id, &vault_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_erc20_on_eos_eth_tx_info_to_metadata() {
        let info = get_sample_erc20_on_eos_eth_tx_info();
        let result = info.to_metadata();
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_erc20_on_eos_eth_tx_info_to_metadata_bytes() {
        let info = get_sample_erc20_on_eos_eth_tx_info();
        let result = info.to_metadata_bytes().unwrap();
        let expected_result = "0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008002e7261c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000810029e0ad25c43c8000000000000000000000000000000000000000000000000";
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_subtract_amount_from_erc20_on_eos_eth_tx_info() {
        let info = get_sample_erc20_on_eos_eth_tx_info();
        let subtrahend = U256::from(1);
        let expected_result = U256::from_dec_str("1336999999999").unwrap();
        let result = info.subtract_amount(subtrahend).unwrap().amount;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_subtract_too_large_amount_from_erc20_on_eos_eth_tx_info() {
        let info = get_sample_erc20_on_eos_eth_tx_info();
        let expected_err = "Cannot subtract amount from `Erc20OnEosEthTxInfo`: subtrahend too large!".to_string();
        let subtrahend = info.amount + 1;
        match info.subtract_amount(subtrahend) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error received!"),
        };
    }

    #[test]
    fn should_calculate_fee_in_erc20_on_eos_eth_tx_info() {
        let basis_points = 25;
        let info = get_sample_erc20_on_eos_eth_tx_info();
        let expected_result = U256::from_dec_str("3342500000").unwrap();
        let result = info.calculate_fee(basis_points).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fees_in_erc20_on_eos_eth_tx_infos() {
        let infos = get_sample_erc20_on_eos_eth_tx_infos();
        let expected_fee = U256::from_dec_str("3342500000").unwrap();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let result = infos.get_fees(&dictionary).unwrap();
        let expected_addresses = [
            EthAddress::from_slice(&hex::decode("32ef9e9a622736399db5ee78a68b258dadbb4353").unwrap()),
            EthAddress::from_slice(&hex::decode("32ef9e9a622736399db5ee78a68b258dadbb4353").unwrap()),
        ];
        assert_eq!(result.len(), infos.len());
        result.iter().enumerate().for_each(|(i, (address, fee))| {
            assert_eq!(*fee, expected_fee);
            assert_eq!(*address, expected_addresses[i])
        });
    }
}
