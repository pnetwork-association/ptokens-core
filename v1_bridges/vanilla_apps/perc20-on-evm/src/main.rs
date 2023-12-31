#[macro_use]
extern crate log;
#[macro_use]
extern crate paste;
#[macro_use]
extern crate common_docopt_macros;

mod get_cli_args;
mod usage_info;

use common::AppError;
#[cfg(feature = "file-logger")]
use common_file_logger::init_logger;
#[cfg(feature = "json-rpc")]
use common_jsonrpc_db::get_db;
#[cfg(feature = "rocks-db")]
use common_rocksdb_database::get_db;
#[cfg(feature = "stderr-logger")]
use common_stderr_logger::init_logger;
use erc20_on_evm::{
    debug_add_debug_signer,
    debug_add_dictionary_entry,
    debug_add_multiple_debug_signers,
    debug_get_add_supported_token_tx,
    debug_get_all_db_keys,
    debug_get_erc20_on_evm_vault_migration_tx,
    debug_get_erc20_on_evm_vault_single_migration_tx,
    debug_get_key_from_db,
    debug_get_remove_supported_token_tx,
    debug_remove_debug_signer,
    debug_remove_dictionary_entry,
    debug_reprocess_eth_block,
    debug_reprocess_eth_block_with_fee_accrual,
    debug_reprocess_eth_block_with_nonce,
    debug_reprocess_evm_block,
    debug_reprocess_evm_block_with_fee_accrual,
    debug_reprocess_evm_block_with_nonce,
    debug_reset_eth_chain,
    debug_reset_evm_chain,
    debug_set_accrued_fees_in_dictionary,
    debug_set_eth_account_nonce,
    debug_set_eth_gas_price,
    debug_set_evm_account_nonce,
    debug_set_evm_gas_price,
    debug_set_fee_basis_points,
    debug_set_key_in_db_to_value,
    debug_withdraw_fees_and_save_in_db,
    get_enclave_state,
    get_latest_block_numbers,
    maybe_add_vault_contract_address,
    maybe_initialize_eth_core,
    maybe_initialize_evm_core,
    sign_ascii_msg_with_eth_key_with_no_prefix,
    sign_ascii_msg_with_evm_key_with_no_prefix,
    sign_hex_msg_with_eth_key_with_prefix,
    sign_hex_msg_with_evm_key_with_prefix,
    submit_eth_block_to_core,
    submit_evm_block_to_core,
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
    let db = get_db()?;
    init_logger()?;
    let cli_args = get_cli_args(USAGE_INFO)?;
    Ok(match cli_args {
        CliArgs {
            cmd_debugMigrateSingle: true,
            ..
        } => {
            info!("✔ Debug migrating single token from vault..");
            Ok(debug_get_erc20_on_evm_vault_single_migration_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.arg_tokenAddress,
                &cli_args.flag_sig,
            )?)
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
            cmd_initializeEth: true,
            ..
        } => {
            info!("✔ Initializing ETH core...");
            Ok(maybe_initialize_eth_core(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_chainId,
                cli_args.flag_gasPrice,
                cli_args.flag_confs,
                &cli_args.flag_vaultAddress,
            )?)
        },
        CliArgs {
            cmd_initializeEvm: true,
            ..
        } => {
            info!("✔ Initializing EVM core...");
            Ok(maybe_initialize_evm_core(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_chainId,
                cli_args.flag_gasPrice,
                cli_args.flag_confs,
            )?)
        },
        CliArgs {
            cmd_getEnclaveState: true,
            ..
        } => {
            info!("✔ Getting core state...");
            Ok(get_enclave_state(&db)?)
        },
        CliArgs {
            cmd_debugMigrateContract: true,
            ..
        } => {
            info!("✔ Debug getting `migrate` transaction...");
            Ok(debug_get_erc20_on_evm_vault_migration_tx(
                &db,
                &cli_args.arg_ethAddress,
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
            cmd_getLatestBlockNumbers: true,
            ..
        } => {
            info!("✔ Maybe getting block numbers...");
            Ok(get_latest_block_numbers(&db)?)
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
            cmd_debugReprocessEthBlock: true,
            ..
        } => {
            info!("✔ Debug reprocessing ETH block...");
            Ok(debug_reprocess_eth_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEthBlockWithNonce: true,
            ..
        } => {
            info!("✔ Debug reprocessing ETH block with nonce...");
            Ok(debug_reprocess_eth_block_with_nonce(
                &db,
                &cli_args.arg_blockJson,
                cli_args.arg_nonce,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEvmBlock: true,
            ..
        } => {
            info!("✔ Debug reprocessing EVM block...");
            Ok(debug_reprocess_evm_block(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEvmBlockWithNonce: true,
            ..
        } => {
            info!("✔ Debug reprocessing EVM block with nonce...");
            Ok(debug_reprocess_evm_block_with_nonce(
                &db,
                &cli_args.arg_blockJson,
                cli_args.arg_nonce,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEthBlockWithFeeAccrual: true,
            ..
        } => {
            info!("✔ Debug reprocessing ETH block with fee accrual...");
            Ok(debug_reprocess_eth_block_with_fee_accrual(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugReprocessEvmBlockWithFeeAccrual: true,
            ..
        } => {
            info!("✔ Debug reprocessing EVM block with fee accrual...");
            Ok(debug_reprocess_evm_block_with_fee_accrual(
                &db,
                &cli_args.arg_blockJson,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_submitEthBlock: true,
            ..
        } => {
            info!("✔ Submitting ETH block to core...");
            Ok(submit_eth_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_submitEvmBlock: true,
            ..
        } => {
            info!("✔ Submitting EVM block to core...");
            Ok(submit_evm_block_to_core(&db, &cli_args.arg_blockJson)?)
        },
        CliArgs {
            cmd_signHexMsgWithEthKeyWithPrefix: true,
            ..
        } => {
            info!("✔ Signing HEX message with ETH key & ETH-specific prefix...");
            Ok(sign_hex_msg_with_eth_key_with_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_signHexMsgWithEvmKeyWithPrefix: true,
            ..
        } => {
            info!("✔ Signing HEX message with EVM key & ETH-specific prefix...");
            Ok(sign_hex_msg_with_evm_key_with_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_signAsciiMsgWithEthKeyWithNoPrefix: true,
            ..
        } => {
            info!("✔ Signing ASCII message with ETH key & NO prefix...");
            Ok(sign_ascii_msg_with_eth_key_with_no_prefix(&db, &cli_args.arg_message)?)
        },
        CliArgs {
            cmd_signAsciiMsgWithEvmKeyWithNoPrefix: true,
            ..
        } => {
            info!("✔ Signing ASCII message with EVM key & NO prefix...");
            Ok(sign_ascii_msg_with_evm_key_with_no_prefix(&db, &cli_args.arg_message)?)
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
            cmd_debugRemoveDictionaryEntry: true,
            ..
        } => {
            info!("✔ Debug removing dictionary entry...");
            Ok(debug_remove_dictionary_entry(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugAddSupportedToken: true,
            ..
        } => {
            info!("✔ Debug getting `addSupportedToken` signed transaction...");
            Ok(debug_get_add_supported_token_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugRemoveSupportedToken: true,
            ..
        } => {
            info!("✔ Debug getting `removeSupportedToken` signed transaction...");
            Ok(debug_get_remove_supported_token_tx(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_addVaultContractAddress: true,
            ..
        } => {
            info!("✔ Maybe adding vault contract address to DB...");
            Ok(maybe_add_vault_contract_address(&db, &cli_args.arg_ethAddress)?)
        },
        CliArgs {
            cmd_debugSetFeeBasisPoints: true,
            ..
        } => {
            info!("✔ Debug setting fee basis points...");
            Ok(debug_set_fee_basis_points(
                &db,
                &cli_args.arg_ethAddress,
                cli_args.arg_fee,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugWithdrawFees: true,
            ..
        } => {
            info!("✔ Debug withdrawing fees...");
            Ok(debug_withdraw_fees_and_save_in_db(
                &db,
                &cli_args.arg_tokenAddress,
                &cli_args.arg_recipientAddress,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEthGasPrice: true,
            ..
        } => {
            info!("✔ Debug setting ETH gas price...");
            Ok(debug_set_eth_gas_price(
                &db,
                cli_args.arg_gasPrice,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEvmGasPrice: true,
            ..
        } => {
            info!("✔ Debug setting EVM gas price...");
            Ok(debug_set_evm_gas_price(
                &db,
                cli_args.arg_gasPrice,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugResetEthChain: true,
            ..
        } => {
            info!("✔ Debug resetting ETH chain...");
            Ok(debug_reset_eth_chain(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_confs,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugResetEvmChain: true,
            ..
        } => {
            info!("✔ Debug resetting EVM chain...");
            Ok(debug_reset_evm_chain(
                &db,
                &cli_args.arg_blockJson,
                cli_args.flag_confs,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEthAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting ETH account nonce...");
            Ok(debug_set_eth_account_nonce(
                &db,
                cli_args.arg_nonce,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetAccruedFees: true,
            ..
        } => {
            info!("✔ Debug setting accrued fees...");
            Ok(debug_set_accrued_fees_in_dictionary(
                &db,
                &cli_args.arg_ethAddress,
                &cli_args.arg_amount,
                &cli_args.flag_sig,
            )?)
        },
        CliArgs {
            cmd_debugSetEvmAccountNonce: true,
            ..
        } => {
            info!("✔ Debug setting EVM account nonce...");
            Ok(debug_set_evm_account_nonce(
                &db,
                cli_args.arg_nonce,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        CliArgs { flag_version: true, .. } => {
            let app_type = option_env!("CARGO_PKG_NAME").unwrap_or("unknown");
            let app_version = option_env!("CARGO_PKG_VERSION").unwrap_or("unkownn");
            Ok(format!("{{app_type: {app_type}, app_version: {app_version}}}"))
        },
        CliArgs {
            cmd_debugAddDebugSigner: true,
            ..
        } => {
            info!("✔ Debug adding debug signer...");
            Ok(debug_add_debug_signer(
                &db,
                &cli_args.arg_name,
                &cli_args.arg_ethAddress,
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
                &cli_args.arg_ethAddress,
                &CliArgs::core_type(),
                &cli_args.flag_sig,
            )?)
        },
        _ => Err(AppError::Custom(USAGE_INFO.to_string())),
    }?)
}
