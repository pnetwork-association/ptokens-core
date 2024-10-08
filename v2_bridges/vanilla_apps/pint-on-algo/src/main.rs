#[macro_use]
extern crate paste;
#[macro_use]
extern crate log;
#[macro_use]
extern crate common_docopt_macros;

mod get_cli_args;
mod usage_info;

use common::AppError;
#[cfg(feature = "file-logger")]
use common_file_logger::init_logger;
#[cfg(feature = "json-rpc-db")]
use common_jsonrpc_db::get_db;
#[cfg(feature = "rocks-db")]
use common_rocksdb_database::get_db;
#[cfg(feature = "std-err-logger")]
use common_stderr_logger::init_logger;
use int_on_algo::{
    debug_add_debug_signer,
    debug_add_dictionary_entry,
    debug_add_multiple_debug_signers,
    debug_get_add_supported_token_tx,
    debug_get_algo_pay_tx,
    debug_get_all_db_keys,
    debug_get_key_from_db,
    debug_opt_in_to_application,
    debug_opt_in_to_asset,
    debug_remove_debug_signer,
    debug_remove_dictionary_entry,
    debug_reprocess_algo_block,
    debug_reprocess_algo_block_with_nonce,
    debug_reprocess_int_block,
    debug_reset_algo_chain,
    debug_reset_int_chain,
    debug_set_algo_account_nonce,
    debug_set_int_account_nonce,
    debug_set_int_gas_price,
    debug_set_key_in_db_to_value,
    encode_algo_note_metadata,
    get_enclave_state,
    get_latest_block_numbers,
    maybe_initialize_algo_core,
    maybe_initialize_int_core,
    submit_algo_block_to_core,
    submit_algo_blocks_to_core,
    submit_int_block_to_core,
    submit_int_blocks_to_core,
};

use crate::{
    get_cli_args::{get_cli_args, CliArgs},
    usage_info::USAGE_INFO,
};

fn main() {
    match program() {
        Ok(s) => {
            info!("{}", s);
            println!("{}", s);
        },
        Err(e) => {
            error!("{}", e);
            println!("{}", e);
            std::process::exit(1);
        },
    }
}

fn program() -> anyhow::Result<String> {
    init_logger()?;
    let db = get_db()?;
    let cli_args = get_cli_args(USAGE_INFO)?;
    Ok(match cli_args {
        CliArgs {
            cmd_debugAddDebugSigners: true,
            ..
        } => {
            info!("✔ Debug adding mulitple debug signers...");
            Ok(debug_add_multiple_debug_signers(
                &db,
                &cli_args.arg_debugSignersJson,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugGetAllDbKeys: true,
            ..
        } => {
            info!("✔ Debug getting all DB keys...");
            Ok(debug_get_all_db_keys(&db, &cli_args.flag_sig)?)
        },
        CliArgs {
            cmd_debugAddSupportedToken: true,
            ..
        } => {
            info!("✔ Debug getting add supported token tx...");
            Ok(debug_get_add_supported_token_tx(
                &db,
                &cli_args.arg_evmAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetIntGasPrice: true,
            ..
        } => {
            info!("✔ Debug setting INT gas price...");
            Ok(debug_set_int_gas_price(
                &db,
                cli_args.arg_gasPrice,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetKeyInDbToValue: true,
            ..
        } => {
            info!("✔ Debug setting key in db to value...");
            Ok(debug_set_key_in_db_to_value(
                &db,
                &cli_args.arg_key,
                &cli_args.arg_value,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugGetKeyFromDb: true,
            ..
        } => {
            info!("✔ Debug setting key in db to value...");
            Ok(debug_get_key_from_db(
                &db,
                &cli_args.arg_key,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetAlgoAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting ALGO account nonce...");
            Ok(debug_set_algo_account_nonce(
                &db,
                cli_args.arg_nonce,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetIntAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting INT account nonce...");
            Ok(debug_set_int_account_nonce(
                &db,
                cli_args.arg_nonce,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_encodeAlgoNoteMetadata: true,
            ..
        } => {
            info!("✔ Encoding ALGO note metadata...");
            Ok(encode_algo_note_metadata(
                &cli_args.arg_metadataChainId,
                &cli_args.arg_destinationAddress,
                &cli_args.flag_userData,
            )?)
        },
        CliArgs {
            cmd_debugRemoveDictionaryEntry: true,
            ..
        } => {
            info!("✔ Debug removing dictionary entry via EVM address...");
            Ok(debug_remove_dictionary_entry(
                &db,
                &cli_args.arg_evmAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddDictionaryEntry: true,
            ..
        } => {
            info!("✔ Debug adding dictionary entry...");
            Ok(debug_add_dictionary_entry(
                &db,
                &cli_args.arg_entryJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugOptInToAsset: true,
            ..
        } => {
            info!("✔ Getting asset opt-in transaction...");
            Ok(debug_opt_in_to_asset(
                &db,
                cli_args.arg_assetId,
                cli_args.arg_firstValid,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugOptInToApp: true,
            ..
        } => {
            info!("✔ Getting application opt-in transaction...");
            Ok(debug_opt_in_to_application(
                &db,
                cli_args.arg_appId,
                cli_args.arg_firstValid,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugResetIntChain: true,
            ..
        } => {
            info!("✔ Debug resetting INT chain...");
            Ok(debug_reset_int_chain(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_confs,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugResetAlgoChain: true,
            ..
        } => {
            info!("✔ Debug resetting ALGO chain...");
            Ok(debug_reset_algo_chain(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_confs,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_getEnclaveState: true,
            ..
        } => {
            info!("✔ Getting enclave state...");
            Ok(get_enclave_state(&db)?)
        },
        CliArgs {
            cmd_submitAlgoBlock: true,
            ..
        } => {
            info!("✔ Submitting ALGO block to core...");
            Ok(submit_algo_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_submitAlgoBlocks: true,
            ..
        } => {
            info!("✔ Submitting ALGO blocks to core...");
            Ok(submit_algo_blocks_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_debugReprocessAlgoBlock: true,
            ..
        } => {
            info!("✔ Debug reprocessing ALGO block...");
            Ok(debug_reprocess_algo_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessAlgoBlockWithNonce: true,
            ..
        } => {
            info!("✔ Debug reprocessing ALGO block with nonce...");
            Ok(debug_reprocess_algo_block_with_nonce(
                &db,
                &cli_args.arg_blockJson,
                cli_args.arg_nonce,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessIntBlock: true,
            ..
        } => {
            info!("✔ Debug reprocessing INT block...");
            Ok(debug_reprocess_int_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_submitIntBlocks: true,
            ..
        } => {
            info!("✔ Submitting INT blocks to core...");
            Ok(submit_int_blocks_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_submitIntBlock: true,
            ..
        } => {
            info!("✔ Submitting INT block to core...");
            Ok(submit_int_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_initializeAlgo: true,
            ..
        } => {
            info!("✔ Initializing ALGO core...");
            Ok(maybe_initialize_algo_core(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_genesisId,
                cli_args.flag_fee,
                cli_args.flag_confs,
                cli_args.flag_appId,
            )?)
        },
        CliArgs {
            cmd_getLatestBlockNumbers: true,
            ..
        } => {
            info!("✔ Getting latest block numbers...");
            Ok(get_latest_block_numbers(&db)?)
        },
        CliArgs {
            cmd_initializeInt: true,
            ..
        } => {
            info!("✔ Initializing INT core...");
            Ok(maybe_initialize_int_core(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_chainId,
                cli_args.flag_gasPrice,
                cli_args.flag_confs,
                &cli_args.flag_vaultAddress,
                &cli_args.flag_routerAddress,
            )?)
        },
        CliArgs {
            cmd_debugAlgoPayTx: true,
            ..
        } => {
            info!("✔ Debug getting ALGO pay tx...");
            Ok(debug_get_algo_pay_tx(
                &db,
                cli_args.arg_firstValid,
                &cli_args.flag_genesisId,
                cli_args.flag_fee,
                &cli_args.arg_receiver,
                &cli_args.flag_note,
                cli_args.arg_amount,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddDebugSigner: true,
            ..
        } => {
            info!("✔ Debug adding debug signer...");
            Ok(debug_add_debug_signer(
                &db,
                &cli_args.arg_name,
                &cli_args.arg_evmAddress,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugRemoveDebugSigner: true,
            ..
        } => {
            info!("✔ Debug removing debug signer...");
            Ok(debug_remove_debug_signer(
                &db,
                &cli_args.arg_evmAddress,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs { flag_version: true, .. } => {
            let app_type = option_env!("CARGO_PKG_NAME").unwrap_or("unknown");
            let app_version = option_env!("CARGO_PKG_VERSION").unwrap_or("unkownn");
            Ok(format!("{{app_type: {app_type}, app_version: {app_version}}}"))
        },
        _ => Err(AppError::Custom(USAGE_INFO.to_string())),
    }?)
}
