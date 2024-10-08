#[macro_use]
extern crate log;
#[macro_use]
extern crate paste;
#[macro_use]
extern crate common_docopt_macros;

mod get_cli_args;
mod usage_info;

use btc_on_int::{
    debug_add_debug_signer,
    debug_add_multiple_debug_signers,
    debug_add_multiple_utxos,
    debug_clear_all_utxos,
    debug_consolidate_utxos,
    debug_consolidate_utxos_to_address,
    debug_get_all_db_keys,
    debug_get_child_pays_for_parent_btc_tx,
    debug_get_key_from_db,
    debug_get_signed_erc777_change_pnetwork_tx,
    debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
    debug_get_signed_erc777_proxy_change_pnetwork_tx,
    debug_maybe_add_utxo_to_db,
    debug_remove_debug_signer,
    debug_remove_utxo,
    debug_reprocess_btc_block,
    debug_reprocess_btc_block_with_nonce,
    debug_reprocess_int_block,
    debug_reset_int_chain,
    debug_set_btc_account_nonce,
    debug_set_btc_fee,
    debug_set_int_account_nonce,
    debug_set_int_gas_price,
    debug_set_key_in_db_to_value,
    get_all_utxos,
    get_enclave_state,
    get_latest_block_numbers,
    maybe_initialize_btc_core,
    maybe_initialize_int_core,
    sign_ascii_msg_with_int_key_with_no_prefix,
    sign_hex_msg_with_int_key_with_prefix,
    submit_btc_block_to_core,
    submit_int_block_to_core,
    submit_int_blocks_to_core,
};
use common::AppError;
#[cfg(feature = "file-logger")]
use common_file_logger::init_logger;
#[cfg(feature = "json-rpc-db")]
use common_jsonrpc_db::get_db;
#[cfg(feature = "rocks-db")]
use common_rocksdb_database::get_db;
#[cfg(feature = "std-err-logger")]
use common_stderr_logger::init_logger;

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
            cmd_submitIntBlocks: true,
            ..
        } => {
            info!("✔ Submitting INT blocks to core...");
            Ok(submit_int_blocks_to_core(&db, &cli_args.arg_blockJson)?)
        },
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
            cmd_initializeInt: true,
            ..
        } => {
            info!("✔ Initializing INT enclave...");
            Ok(maybe_initialize_int_core(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_chainId,
                cli_args.flag_gasPrice,
                cli_args.flag_confs,
                &cli_args.flag_pTokenAddress,
                &cli_args.flag_routerAddress,
            )?)
        },
        CliArgs {
            cmd_initializeBtc: true,
            ..
        } => {
            info!("✔ Initializing BTC enclave...");
            Ok(maybe_initialize_btc_core(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_fee,
                cli_args.flag_difficulty,
                &cli_args.flag_network,
                cli_args.flag_confs,
            )?)
        },
        CliArgs {
            cmd_debugGetChildPaysForParentTx: true,
            ..
        } => {
            info!("✔ Debug getting `child-pays-for-parent` tx...");
            Ok(debug_get_child_pays_for_parent_btc_tx(
                &db,
                cli_args.flag_fee,
                &cli_args.arg_txId,
                cli_args.arg_vOut,
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
            cmd_debugGetAllDbKeys: true,
            ..
        } => {
            info!("✔ Debug getting all DB keys....");
            Ok(debug_get_all_db_keys(&db, &cli_args.flag_sig)?)
        },
        CliArgs {
            cmd_getLatestBlockNumbers: true,
            ..
        } => {
            info!("✔ Maybe getting block numbers...");
            Ok(get_latest_block_numbers(&db)?)
        },
        CliArgs {
            cmd_getAllUtxos: true, ..
        } => {
            info!("✔ Getting all UTXOs from the database...");
            Ok(get_all_utxos(&db)?)
        },
        CliArgs {
            cmd_debugClearAllUtxos: true,
            ..
        } => {
            info!("✔ Debug clearing all UTXOs from the database...");
            Ok(debug_clear_all_utxos(&db, &CliArgs::core_type(), &cli_args.flag_sig)?)
        },
        CliArgs {
            cmd_debugGetKeyFromDb: true,
            ..
        } => {
            info!("✔ Maybe getting a key from the database...");
            Ok(debug_get_key_from_db(
                &db,
                &cli_args.arg_key,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugMaybeAddUtxoToDb: true,
            ..
        } => {
            info!("✔ Debug maybe adding UTXO to db...");
            Ok(debug_maybe_add_utxo_to_db(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddUtxos: true,
            ..
        } => {
            info!("✔ Debug adding multiple UTXOs...");
            Ok(debug_add_multiple_utxos(
                &db,
                &cli_args.arg_utxosJson,
                &CliArgs::core_type(),
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
            cmd_debugReprocessBtcBlock: true,
            ..
        } => {
            info!("✔ Debug reprocessing BTC block...");
            Ok(debug_reprocess_btc_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessBtcBlockWithNonce: true,
            ..
        } => {
            info!("✔ Debug reprocessing BTC block with nonce...");
            Ok(debug_reprocess_btc_block_with_nonce(
                &db,
                &cli_args.arg_blockJson,
                cli_args.arg_nonce,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_submitIntBlock: true,
            ..
        } => {
            info!("✔ Submitting INT block to enclave...");
            Ok(submit_int_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_submitBtcBlock: true,
            ..
        } => {
            info!("✔ Submitting BTC block to enclave...");
            Ok(submit_btc_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_debugRemoveUtxo: true,
            ..
        } => {
            info!("✔ Debug removing UTXO...");
            Ok(debug_remove_utxo(
                &db,
                &cli_args.arg_txId,
                cli_args.arg_vOut,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_signHexMsgWithIntKeyWithPrefix: true,
            ..
        } => {
            info!("✔ Signing HEX message with INT key & INT-specific prefix...");
            Ok(sign_hex_msg_with_int_key_with_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_debugConsolidateUtxos: true,
            ..
        } => {
            info!("✔ Debug consolidating utxos...");
            Ok(debug_consolidate_utxos(
                &db,
                cli_args.flag_fee,
                cli_args.arg_numUtxos,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugConsolidateUtxosToAddress: true,
            ..
        } => {
            info!("✔ Debug consolidating UTXOS...");
            Ok(debug_consolidate_utxos_to_address(
                &db,
                cli_args.flag_fee,
                cli_args.arg_numUtxos,
                &cli_args.arg_address,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugErc777ChangePNetwork: true,
            ..
        } => {
            info!("✔ Debug getting `changePNetwork` tx...");
            Ok(debug_get_signed_erc777_change_pnetwork_tx(
                &db,
                &cli_args.arg_address,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_signMessageWithIntKey: true,
            ..
        }
        | CliArgs {
            cmd_signAsciiMsgWithIntKeyWithNoPrefix: true,
            ..
        } => {
            info!("✔ Signing ASCII message with INT key & NO prefix...");
            Ok(sign_ascii_msg_with_int_key_with_no_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_debugSetKeyInDbToValue: true,
            ..
        } => {
            info!("✔ Setting a key in the database to a value...");
            Ok(debug_set_key_in_db_to_value(
                &db,
                &cli_args.arg_key,
                &cli_args.arg_value,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugErc777ProxyChangePNetwork: true,
            ..
        } => {
            info!("✔ Debug getting `changePNetwork` in the proxy tx...");
            Ok(debug_get_signed_erc777_proxy_change_pnetwork_tx(
                &db,
                &cli_args.arg_address,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugErc777ProxyChangePNetworkByProxy: true,
            ..
        } => {
            info!("✔ Debug getting `changePNetworkByProxy` tx...");
            Ok(debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx(
                &db,
                &cli_args.arg_address,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetIntGasPrice: true,
            ..
        } => {
            info!("✔ Debug setting INT gas price to {} Wei..", cli_args.arg_wei);
            Ok(debug_set_int_gas_price(
                &db,
                cli_args.arg_wei,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetBtcFee: true,
            ..
        } => {
            info!("✔ Debug setting BTC fee to {} Satoshis-per-byte...", cli_args.arg_fee);
            Ok(debug_set_btc_fee(
                &db,
                cli_args.arg_fee,
                &CliArgs::core_type(),
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
            cmd_debugSetBtcAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting BTC account nonce...");
            Ok(debug_set_btc_account_nonce(
                &db,
                cli_args.arg_nonce,
                &CliArgs::core_type(),
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
                &cli_args.arg_address,
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
                &cli_args.arg_address,
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
