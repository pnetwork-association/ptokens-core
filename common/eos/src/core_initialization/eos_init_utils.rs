use std::str::FromStr;

use common::{
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryJson},
    traits::DatabaseInterface,
    types::{Bytes, NoneError, Result},
    AppError,
};
use common_chain_ids::EosChainId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    eos_constants::EOS_CORE_IS_INITIALIZED_JSON,
    eos_crypto::eos_private_key::EosPrivateKey,
    eos_database_utils::EosDbUtils,
    eos_global_sequences::ProcessedGlobalSequences,
    eos_incremerkle::{Incremerkle, Incremerkles},
    eos_producer_schedule::EosProducerScheduleV2,
    eos_submission_material::EosSubmissionMaterial,
    eos_types::{Checksum256s, EosBlockHeaderJson, EosKnownSchedules},
    eos_utils::convert_hex_to_checksum256,
    protocol_features::{EnabledFeatures, WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH},
    validate_signature::check_block_signature_is_valid,
    EosState,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosInitJson {
    pub block: EosBlockHeaderJson,
    pub blockroot_merkle: Vec<String>,
    pub active_schedule: EosProducerScheduleV2,
    pub maybe_protocol_features_to_enable: Option<Vec<String>>,
    pub eos_eth_token_dictionary: Option<EosEthTokenDictionaryJson>,
    pub erc20_on_eos_token_dictionary: Option<EosEthTokenDictionaryJson>,
}

impl FromStr for EosInitJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        EosInitJson::from_json_string(s)
    }
}

impl EosInitJson {
    pub fn from_json_string(json_string: &str) -> Result<Self> {
        // NOTE: This inefficient way allows us ot deail with init block with EITHER v1 || v2 schedules.
        // The inefficiency is moot however since this function only gets called once in a core's lifetime
        #[derive(Deserialize)]
        pub struct InterimInitJson {
            pub block: EosBlockHeaderJson,
            pub active_schedule: JsonValue,
            pub blockroot_merkle: Vec<String>,
            pub maybe_protocol_features_to_enable: Option<Vec<String>>,
            pub eos_eth_token_dictionary: Option<EosEthTokenDictionaryJson>,
            pub erc20_on_eos_token_dictionary: Option<EosEthTokenDictionaryJson>,
        }
        let interim_init_json = serde_json::from_str::<InterimInitJson>(json_string)?;
        let producer_schedule = EosProducerScheduleV2::from_json(&interim_init_json.active_schedule.to_string())?;
        Ok(EosInitJson {
            block: interim_init_json.block.clone(),
            blockroot_merkle: interim_init_json.blockroot_merkle.clone(),
            active_schedule: producer_schedule,
            maybe_protocol_features_to_enable: interim_init_json.maybe_protocol_features_to_enable,
            eos_eth_token_dictionary: interim_init_json.eos_eth_token_dictionary.clone(),
            erc20_on_eos_token_dictionary: interim_init_json.erc20_on_eos_token_dictionary,
        })
    }

    #[cfg(test)]
    pub fn validate(&self) {
        use eos_chain::Checksum256;
        let msig_enabled = match &self.maybe_protocol_features_to_enable {
            None => false,
            Some(features) => features.contains(&WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH.to_string()),
        };
        let block_header = EosSubmissionMaterial::parse_eos_block_header_from_json(&self.block).unwrap();
        let blockroot_merkle = self
            .blockroot_merkle
            .iter()
            .map(convert_hex_to_checksum256)
            .collect::<Result<Vec<Checksum256>>>()
            .unwrap();
        let producer_signature = self.block.producer_signature.clone();
        let incremerkle = Incremerkle::new((block_header.block_num() - 1).into(), blockroot_merkle);
        let block_mroot = incremerkle.get_root().to_bytes().to_vec();
        debug!("block mroot: {}", hex::encode(&block_mroot));
        if check_block_signature_is_valid(
            msig_enabled,
            &block_mroot,
            &producer_signature,
            &block_header,
            &self.active_schedule,
        )
        .is_err()
        {
            panic!("Could not validate init block!");
        }
    }
}

pub fn maybe_enable_protocol_features_and_return_state<'a, D: DatabaseInterface>(
    maybe_protocol_features_to_enable: &Option<Vec<String>>,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    match maybe_protocol_features_to_enable {
        None => {
            info!("✘ No protocol features to enable: Skipping!");
            Ok(state)
        },
        Some(feature_hash_strings) => {
            info!("✔ Maybe enabling {} protocol features...", feature_hash_strings.len());
            let mut feature_hashes = feature_hash_strings
                .iter()
                .map(|hex| Ok(hex::decode(hex)?))
                .collect::<Result<Vec<Bytes>>>()?;
            EnabledFeatures::init()
                .enable_multi(&state.eos_db_utils, &mut feature_hashes)
                .and_then(|features| state.add_enabled_protocol_features(features))
        },
    }
}

pub fn test_block_validation_and_return_state<'a, D: DatabaseInterface>(
    block_json: &EosBlockHeaderJson,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    if cfg!(feature = "non-validating") {
        info!("✔ Skipping EOS init block validation check!");
        Ok(state)
    } else {
        info!("checking block validation passes...");
        check_block_signature_is_valid(
            state.enabled_protocol_features.is_enabled(
                hex::decode(WTMSIG_BLOCK_SIGNATURE_FEATURE_HASH)
                    .unwrap_or_default()
                    .as_ref(),
            ),
            Incremerkles::get_from_db(&EosDbUtils::new(state.db))?
                .get_incremerkle_for_block_number(block_json.block_num)?
                .get_root()
                .to_bytes()
                .as_ref(),
            &block_json.producer_signature,
            &EosSubmissionMaterial::parse_eos_block_header_from_json(block_json)?,
            &state
                .eos_db_utils
                .get_eos_schedule_from_db(block_json.schedule_version)?,
        )
        .and(Ok(state))
    }
}

pub fn generate_and_put_incremerkle_in_db<D: DatabaseInterface>(
    db_utils: &EosDbUtils<D>,
    init_json: &EosInitJson,
) -> Result<()> {
    info!(
        "generating and putting new incremerkle in db for block num {}...",
        init_json.block.block_num
    );

    let incremerkle = Incremerkle::new(
        init_json.block.block_num,
        init_json
            .blockroot_merkle
            .iter()
            .map(convert_hex_to_checksum256)
            .collect::<Result<Checksum256s>>()?,
    );

    let incremerkles = Incremerkles::new(vec![incremerkle]);
    incremerkles.put_in_db(db_utils)?;
    Ok(())
}

pub fn generate_and_put_incremerkle_in_db_and_return_state<'a, D: DatabaseInterface>(
    init_json: &EosInitJson,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    generate_and_put_incremerkle_in_db(&state.eos_db_utils, init_json).and(Ok(state))
}

pub fn put_eos_latest_block_info_in_db<D: DatabaseInterface>(
    db_utils: &EosDbUtils<D>,
    block_json: &EosBlockHeaderJson,
) -> Result<()> {
    info!(
        "✔ Putting latest block number '{}' & ID '{}' into db...",
        &block_json.block_num, &block_json.block_id
    );
    db_utils
        .put_eos_last_seen_block_num_in_db(block_json.block_num)
        .and_then(|_| {
            db_utils.put_eos_last_seen_block_id_in_db(&convert_hex_to_checksum256(block_json.block_id.clone())?)
        })
}

pub fn put_eos_latest_block_info_in_db_and_return_state<'a, D: DatabaseInterface>(
    block_json: &EosBlockHeaderJson,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    put_eos_latest_block_info_in_db(&state.eos_db_utils, block_json).and(Ok(state))
}

pub fn put_eos_known_schedule_in_db_and_return_state<'a, D: DatabaseInterface>(
    schedule: &EosProducerScheduleV2,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    info!("✔ Putting EOS known schedule into db...");
    state
        .eos_db_utils
        .put_eos_known_schedules_in_db(&EosKnownSchedules::new(schedule.version))
        .and(Ok(state))
}

pub fn put_eos_schedule_in_db_and_return_state<'a, D: DatabaseInterface>(
    schedule: &EosProducerScheduleV2,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    info!("✔ Putting EOS schedule into db...");
    state.eos_db_utils.put_eos_schedule_in_db(schedule).and(Ok(state))
}

pub fn generate_and_save_eos_keys_and_return_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Generating EOS keys & putting into db...");
    let private_key = EosPrivateKey::generate_random()?;
    state
        .eos_db_utils
        .put_eos_public_key_in_db(&private_key.to_public_key())
        .and_then(|_| private_key.write_to_db(state.db))
        .and(Ok(state))
}

pub fn get_eos_init_output<D: DatabaseInterface>(_state: EosState<D>) -> Result<String> {
    Ok(EOS_CORE_IS_INITIALIZED_JSON.to_string())
}

pub fn put_eos_account_name_in_db_and_return_state<'a, D: DatabaseInterface>(
    account_name: &str,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    info!("✔ Putting EOS account name '{}' into db...", account_name);
    state
        .eos_db_utils
        .put_eos_account_name_in_db(account_name)
        .and(Ok(state))
}

pub fn initialize_eos_account_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Initializing EOS account nonce to 0 in db...");
    state.eos_db_utils.put_eos_account_nonce_in_db(0).and(Ok(state))
}

pub fn put_eos_token_symbol_in_db_and_return_state<'a, D: DatabaseInterface>(
    token_symbol: &str,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    info!("✔ Putting EOS token symbol '{}' into db...", token_symbol);
    state
        .eos_db_utils
        .put_eos_token_symbol_in_db(token_symbol)
        .and(Ok(state))
}

pub fn put_empty_processed_tx_ids_in_db_and_return_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Initializing EOS processed tx ids & putting into db...");
    ProcessedGlobalSequences::new(vec![]).put_in_db(state.db).and(Ok(state))
}

pub fn put_eos_chain_id_in_db_and_return_state<'a, D: DatabaseInterface>(
    chain_id: &str,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    info!("✔ Putting EOS chain ID '{}' into db...", chain_id);
    state
        .eos_db_utils
        .put_eos_chain_id_in_db(&EosChainId::from_str(chain_id)?)
        .and(Ok(state))
}

pub fn maybe_put_eos_eth_token_dictionary_in_db_and_return_state<'a, D: DatabaseInterface>(
    init_json: &EosInitJson,
    state: EosState<'a, D>,
) -> Result<EosState<'a, D>> {
    info!("✔ Maybe putting `EOS_ETH` token dictionary in db...");
    if init_json.erc20_on_eos_token_dictionary.is_some() && init_json.eos_eth_token_dictionary.is_some() {
        return Err("Found both `erc20-on-eos` & `eos_eth` dictionaries in json - please provide only one!".into());
    };
    let json = if init_json.erc20_on_eos_token_dictionary.is_some() {
        init_json
            .erc20_on_eos_token_dictionary
            .as_ref()
            .ok_or(NoneError("✘ Could not unwrap `erc20_on_eos_token_dictionary`!"))?
    } else if init_json.eos_eth_token_dictionary.is_some() {
        init_json
            .eos_eth_token_dictionary
            .as_ref()
            .ok_or(NoneError("✘ Could not unwrap `eos_eth_token_dictionary`!"))?
    } else {
        info!("✔ No `eos_eth_token_dictionary` in `init-json` ∴ doing nothing!");
        return Ok(state);
    };
    info!("✔ `EosEthTokenDictionary` found in `init-json` ∴ putting it in db...");
    EosEthTokenDictionary::from_json(json)
        .and_then(|dict| dict.save_to_db(state.db))
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eos_test_utils::{
        get_j3_init_json_n,
        get_mainnet_init_json_n,
        get_sample_init_block_with_v1_schedule,
        get_sample_mainnet_init_json_with_eos_eth_token_dictionary,
        NUM_J3_INIT_SAMPLES,
        NUM_MAINNET_INIT_SAMPLES,
    };

    #[test]
    fn should_validate_jungle_3_init_blocks() {
        [0; NUM_J3_INIT_SAMPLES].iter().enumerate().for_each(|(i, _)| {
            println!("Validating jungle 3 init block #{}...", i + 1);
            get_j3_init_json_n(i + 1).unwrap().validate();
        });
    }

    #[test]
    fn should_validate_mainnet_init_blocks() {
        [0; NUM_MAINNET_INIT_SAMPLES].iter().enumerate().for_each(|(i, _)| {
            println!("Validating mainnet init block #{}...", i + i);
            get_mainnet_init_json_n(i + 1).unwrap().validate();
        });
    }

    #[test]
    fn should_parse_init_json_with_eos_eth_token_dictionary() {
        let init_json_with_eos_eth_token_dictionary = get_sample_mainnet_init_json_with_eos_eth_token_dictionary();
        assert!(init_json_with_eos_eth_token_dictionary.is_ok());
    }

    #[test]
    fn should_get_init_json_from_init_block_with_v1_schedule() {
        let fio_init_json_string = get_sample_init_block_with_v1_schedule().unwrap();
        let result = EosInitJson::from_json_string(&fio_init_json_string);
        assert!(result.is_ok());
    }
}
