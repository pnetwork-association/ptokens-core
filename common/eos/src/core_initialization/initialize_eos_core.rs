use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};

use crate::{
    core_initialization::eos_init_utils::{
        generate_and_put_incremerkle_in_db_and_return_state,
        generate_and_save_eos_keys_and_return_state,
        get_eos_init_output,
        initialize_eos_account_nonce_in_db_and_return_state,
        maybe_enable_protocol_features_and_return_state,
        maybe_put_eos_eth_token_dictionary_in_db_and_return_state,
        put_empty_processed_tx_ids_in_db_and_return_state,
        put_eos_account_name_in_db_and_return_state,
        put_eos_chain_id_in_db_and_return_state,
        put_eos_known_schedule_in_db_and_return_state,
        put_eos_latest_block_info_in_db_and_return_state,
        put_eos_schedule_in_db_and_return_state,
        put_eos_token_symbol_in_db_and_return_state,
        test_block_validation_and_return_state,
        EosInitJson,
    },
    eos_constants::EOS_CORE_IS_INITIALIZED_JSON,
    eos_database_transactions::{end_eos_db_transaction_and_return_state, start_eos_db_transaction_and_return_state},
    EosState,
};

pub fn initialize_eos_core_inner<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    maybe_account_name: Option<&str>,
    maybe_token_symbol: Option<&str>,
    eos_init_json: &str,
    is_native: bool,
) -> Result<String> {
    let init_json = EosInitJson::from_json_string(eos_init_json)?;
    info!("✔ Initializing core for EOS...");
    start_eos_db_transaction_and_return_state(EosState::init(db))
        .and_then(put_empty_processed_tx_ids_in_db_and_return_state)
        .and_then(|state| put_eos_chain_id_in_db_and_return_state(chain_id, state))
        .and_then(|state| match maybe_account_name {
            Some(account_name) => put_eos_account_name_in_db_and_return_state(account_name, state),
            None => Ok(state),
        })
        .and_then(|state| match maybe_token_symbol {
            Some(token_symbol) => put_eos_token_symbol_in_db_and_return_state(token_symbol, state),
            None => Ok(state),
        })
        .and_then(|state| put_eos_known_schedule_in_db_and_return_state(&init_json.active_schedule, state))
        .and_then(|state| put_eos_schedule_in_db_and_return_state(&init_json.active_schedule, state))
        .and_then(|state| {
            if cfg!(feature = "spring_1-0") {
                put_eos_latest_block_info_in_db_and_return_state(&init_json.block, state)
            } else {
                generate_and_put_incremerkle_in_db_and_return_state(&init_json, state)
            }
        })
        .and_then(|state| {
            maybe_enable_protocol_features_and_return_state(&init_json.maybe_protocol_features_to_enable, state)
        })
        .and_then(|state| maybe_put_eos_eth_token_dictionary_in_db_and_return_state(&init_json, state))
        .and_then(|state| test_block_validation_and_return_state(&init_json.block, state))
        .and_then(generate_and_save_eos_keys_and_return_state)
        .and_then(initialize_eos_account_nonce_in_db_and_return_state)
        .and_then(|state| {
            if is_native {
                CoreType::initialize_native_core(state.eos_db_utils.get_db())?
            } else {
                CoreType::initialize_host_core(state.eos_db_utils.get_db())?
            };
            Ok(state)
        })
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_init_output)
}

pub fn initialize_eos_core<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    maybe_account_name: Option<&str>,
    maybe_token_symbol: Option<&str>,
    eos_init_json: &str,
    is_native: bool,
) -> Result<String> {
    initialize_eos_core_inner(
        db,
        chain_id,
        maybe_account_name,
        maybe_token_symbol,
        eos_init_json,
        is_native,
    )
}

/// # Maybe Initialize EOS Core With EOS Account & Symbol
///
/// This function first checks to see if the EOS side of a core has been initialized, and will
/// initialize it if not. The initialization procedure takes as its input a database, the
/// `chain_id` of the desired EOS chain, an account name for the EOS smart-contract, the token
/// symbol of the EOS smart-contract and an EOS init JSON string.
///
/// The EOS init JSON string is of the format:
///
/// ```no_compile
/// {
///    block: EosBlockHeaderJson,
///    blockroot_merkle: [txHash...],
///    active_schedule: EosProducerScheduleJsonV2,
///    maybe_protocol_features_to_enable: [protocolFeatureHash...],
///    erc20_on_eos_token_dictionary: EosEthTokenDictionaryJson,
/// }
/// ```
pub fn maybe_initialize_eos_core_with_eos_account_and_symbol<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    account_name: &str,
    token_symbol: &str,
    eos_init_json: &str,
    is_native: bool,
) -> Result<String> {
    info!("✔ Maybe initializing EOS core...");
    let core_is_initialized = if is_native {
        CoreType::native_core_is_initialized(db)
    } else {
        CoreType::host_core_is_initialized(db)
    };
    if core_is_initialized {
        Ok(EOS_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        initialize_eos_core(
            db,
            chain_id,
            Some(account_name),
            Some(token_symbol),
            eos_init_json,
            is_native,
        )
    }
}

/// # Maybe Initialize EOS Core Without EOS Account or Symbol
///
/// This function first checks to see if the EOS side of a core has been initialized, and will
/// initialize it if not. The initialization procedure takes as its input a database, the
/// `chain_id` of the desired EOS chain and an EOS init JSON string. This version of the init
/// function does not requite an EOS account name or symbol, and is therefore useful for EOS
/// related instances that use a token dictionary to define the bridges instead.
///
/// The EOS init JSON string is of the format:
///
/// ```no_compile
/// {
///    block: EosBlockHeaderJson,
///    blockroot_merkle: [txHash...],
///    active_schedule: EosProducerScheduleJsonV2,
///    maybe_protocol_features_to_enable: [protocolFeatureHash...],
///    erc20_on_eos_token_dictionary: EosEthTokenDictionaryJson,
/// }
/// ```
pub fn maybe_initialize_eos_core_without_eos_account_or_symbol<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    eos_init_json: &str,
    is_native: bool,
) -> Result<String> {
    info!("✔ Maybe initializing EOS core...");
    let core_is_initialized = if is_native {
        CoreType::native_core_is_initialized(db)
    } else {
        CoreType::host_core_is_initialized(db)
    };
    if core_is_initialized {
        Ok(EOS_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        initialize_eos_core(db, chain_id, None, None, eos_init_json, is_native)
    }
}

/// # Maybe Initialize EOS Core With EOS Account Without Symbol
///
/// This function first checks to see if the EOS side of a core has been initialized, and will
/// initialize it if not. The initialization procedure takes as its input a database, the
/// `chain_id` of the desired EOS chain and an EOS init JSON string. This version of the init
/// function does not requite an EOS token symbol and is therefore useful for EOS related
/// instances which use a token dictionary to define the bridges instead. The EOS account name is
/// used to define the vault contract on EOS, against which name the incoming action proofs are
/// checked to ensure the correct originating smart-contract.
///
/// The EOS init JSON string is of the format:
///
/// ```no_compile
/// {
///    block: EosBlockHeaderJson,
///    blockroot_merkle: [txHash...],
///    active_schedule: EosProducerScheduleJsonV2,
///    maybe_protocol_features_to_enable: [protocolFeatureHash...],
///    erc20_on_eos_token_dictionary: EosEthTokenDictionaryJson,
/// }
/// ```
pub fn maybe_initialize_eos_core_with_eos_account_without_symbol<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    eos_account_name: &str,
    eos_init_json: &str,
    is_native: bool,
) -> Result<String> {
    info!("✔ Maybe initializing EOS core...");
    let core_is_initialized = if is_native {
        CoreType::native_core_is_initialized(db)
    } else {
        CoreType::host_core_is_initialized(db)
    };
    if core_is_initialized {
        Ok(EOS_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        initialize_eos_core(db, chain_id, Some(eos_account_name), None, eos_init_json, is_native)
    }
}
