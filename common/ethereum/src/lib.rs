mod add_block_and_receipts_to_db;
mod any_sender;
mod append_to_blockchain;
mod calculate_linker_hash;
mod check_parent_exists;
mod core_initialization;
mod eip_1559;
mod eth_block;
mod eth_block_from_json_rpc;
mod eth_constants;
mod eth_contracts;
mod eth_crypto;
mod eth_database_transactions;
mod eth_database_utils;
mod eth_enclave_state;
mod eth_log;
mod eth_macros;
mod eth_message_signer;
mod eth_receipt;
mod eth_receipt_from_json_rpc;
mod eth_receipt_type;
mod eth_state;
mod eth_submission_material;
mod eth_traits;
mod eth_types;
mod eth_utils;
mod increment_eth_account_nonce;
mod increment_evm_account_nonce;
mod increment_int_account_nonce;
mod remove_old_eth_tail_block;
mod remove_receipts_from_canon_block;
mod update_eth_canon_block_hash;
mod update_eth_linker_hash;
mod update_eth_tail_block_hash;
mod update_latest_block_hash;
mod validate_block_in_state;
mod validate_receipts_in_state;
mod vault_using_cores;

pub mod test_utils;

pub use self::{
    add_block_and_receipts_to_db::{
        maybe_add_eth_block_and_receipts_to_db_and_return_state,
        maybe_add_evm_block_and_receipts_to_db_and_return_state,
    },
    any_sender::{RelayTransaction, RelayTransactions},
    append_to_blockchain::append_to_blockchain,
    check_parent_exists::{check_for_parent_of_eth_block_in_state, check_for_parent_of_evm_block_in_state},
    core_initialization::{
        add_eth_block_to_db_and_return_state,
        add_evm_block_to_db_and_return_state,
        generate_and_store_eos_on_eth_contract_address,
        init_v3_host_core,
        init_v3_native_core,
        initialize_eth_core_with_no_contract_tx,
        initialize_eth_core_with_router_contract_and_return_state,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        initialize_evm_core_with_no_contract_tx,
        put_eth_canon_to_tip_length_in_db_and_return_state,
        put_eth_tail_block_hash_in_db_and_return_state,
        put_evm_canon_to_tip_length_in_db_and_return_state,
        put_evm_tail_block_hash_in_db_and_return_state,
        remove_receipts_from_block_in_state,
        set_eth_anchor_block_hash_and_return_state,
        set_eth_canon_block_hash_and_return_state,
        set_eth_latest_block_hash_and_return_state,
        set_evm_anchor_block_hash_and_return_state,
        set_evm_canon_block_hash_and_return_state,
        set_evm_latest_block_hash_and_return_state,
        EthInitializationOutput,
    },
    eth_block::{EthBlock, EthBlockJson},
    eth_block_from_json_rpc::EthBlockJsonFromRpc,
    eth_constants::{
        ETH_ADDRESS_SIZE_IN_BYTES,
        ETH_CORE_IS_INITIALIZED_JSON,
        EVM_CORE_IS_INITIALIZED_JSON,
        MAX_BYTES_FOR_ETH_USER_DATA,
        VALUE_FOR_MINTING_TX,
        ZERO_ETH_VALUE,
    },
    eth_contracts::{
        encode_erc20_vault_add_supported_token_fx_data,
        encode_erc20_vault_migrate_fxn_data,
        encode_erc20_vault_migrate_single_fxn_data,
        encode_erc20_vault_peg_out_fxn_data_with_user_data,
        encode_erc20_vault_peg_out_fxn_data_without_user_data,
        encode_erc20_vault_remove_supported_token_fx_data,
        encode_erc20_vault_set_weth_unwrapper_address_fxn_data,
        encode_erc777_mint_fxn_maybe_with_data,
        encode_erc777_mint_with_no_data_fxn,
        encode_mint_by_proxy_tx_data,
        get_signed_erc777_change_pnetwork_tx,
        get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
        get_signed_erc777_proxy_change_pnetwork_tx,
        Erc20TokenTransferEvent,
        Erc20TokenTransferEvents,
        Erc20VaultPegInEventParams,
        Erc777BurnEvent,
        Erc777RedeemEvent,
        SupportedTopics,
        ToErc20TokenTransferEvent,
        ToWethDepositEvent,
        WethDepositEvent,
        WethDepositEvents,
        ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2,
        ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC,
        ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
        ERC777_REDEEM_EVENT_TOPIC_V2,
        ERC_777_BURN_EVENT_TOPIC,
        ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
        ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
    },
    eth_crypto::{get_signed_minting_tx, EthPrivateKey, EthPublicKey, EthSignature, EthTransaction, EthTransactions},
    eth_database_transactions::{end_eth_db_transaction_and_return_state, start_eth_db_transaction_and_return_state},
    eth_database_utils::{EthDatabaseKeysJson, EthDbUtils, EthDbUtilsExt, EvmDatabaseKeysJson, EvmDbUtils},
    eth_enclave_state::{EthEnclaveState, EvmEnclaveState},
    eth_log::{EthLog, EthLogExt, EthLogs},
    eth_message_signer::{
        sign_ascii_msg_with_eth_key_with_no_prefix,
        sign_ascii_msg_with_eth_key_with_prefix,
        sign_ascii_msg_with_evm_key_with_no_prefix,
        sign_ascii_msg_with_evm_key_with_prefix,
        sign_hex_msg_with_eth_key_with_prefix,
        sign_hex_msg_with_evm_key_with_prefix,
    },
    eth_receipt::{EthReceipt, EthReceiptJson, EthReceipts},
    eth_receipt_from_json_rpc::EthReceiptFromJsonRpc,
    eth_state::{EthState, EthStateCompatible},
    eth_submission_material::{
        parse_eth_submission_material_and_put_in_state,
        parse_eth_submission_material_json_and_put_in_state,
        EthSubmissionMaterial,
        EthSubmissionMaterialJson,
        EthSubmissionMaterialJsons,
        EthSubmissionMaterials,
    },
    eth_traits::{EthSigningCapabilities, EthTxInfoCompatible},
    eth_types::{AnySenderSigningParams, EthSigningParams},
    eth_utils::{
        convert_eth_address_to_string,
        convert_eth_hash_to_string,
        convert_h256_to_bytes,
        convert_h256_to_eth_address,
        convert_h256_to_string,
        convert_hex_strings_to_eth_addresses,
        convert_hex_strings_to_h256s,
        convert_hex_to_eth_address,
        convert_hex_to_h256,
        get_eth_address_from_str,
        get_random_eth_address,
    },
    increment_eth_account_nonce::maybe_increment_eth_account_nonce_and_return_state,
    increment_evm_account_nonce::maybe_increment_evm_account_nonce_and_return_eth_state,
    increment_int_account_nonce::maybe_increment_int_account_nonce_and_return_eth_state,
    remove_old_eth_tail_block::{
        maybe_remove_old_eth_tail_block_and_return_state,
        maybe_remove_old_evm_tail_block_and_return_state,
    },
    remove_receipts_from_canon_block::{
        maybe_remove_receipts_from_eth_canon_block_and_return_state,
        maybe_remove_receipts_from_evm_canon_block_and_return_state,
    },
    update_eth_canon_block_hash::{
        maybe_update_eth_canon_block_hash_and_return_state,
        maybe_update_evm_canon_block_hash_and_return_state,
    },
    update_eth_linker_hash::{
        maybe_update_eth_linker_hash_and_return_state,
        maybe_update_evm_linker_hash_and_return_state,
    },
    update_eth_tail_block_hash::{
        maybe_update_eth_tail_block_hash_and_return_state,
        maybe_update_evm_tail_block_hash_and_return_state,
    },
    update_latest_block_hash::{
        maybe_update_latest_eth_block_hash_and_return_state,
        maybe_update_latest_evm_block_hash_and_return_state,
    },
    validate_block_in_state::{validate_eth_block_in_state, validate_evm_block_in_state},
    validate_receipts_in_state::validate_receipts_in_state,
    vault_using_cores::VaultUsingCores,
};

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate paste;
#[macro_use]
extern crate common;
#[macro_use]
extern crate log;
