use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_chain_ids::EthChainId;
use ethereum_types::Address as EthAddress;

use crate::{
    core_initialization::{
        eth_core_init_utils::{
            add_eth_block_to_db_and_return_state,
            add_evm_block_to_db_and_return_state,
            put_eth_account_nonce_in_db_and_return_state,
            put_eth_any_sender_nonce_in_db_and_return_state,
            put_eth_canon_to_tip_length_in_db_and_return_state,
            put_eth_chain_id_in_db_and_return_state,
            put_eth_gas_price_in_db_and_return_state,
            put_eth_tail_block_hash_in_db_and_return_state,
            put_evm_account_nonce_in_db_and_return_state,
            put_evm_any_sender_nonce_in_db_and_return_state,
            put_evm_canon_to_tip_length_in_db_and_return_state,
            put_evm_chain_id_in_db_and_return_state,
            put_evm_gas_price_in_db_and_return_state,
            put_evm_tail_block_hash_in_db_and_return_state,
            remove_receipts_from_block_in_state,
            set_eth_anchor_block_hash_and_return_state,
            set_eth_canon_block_hash_and_return_state,
            set_eth_latest_block_hash_and_return_state,
            set_evm_anchor_block_hash_and_return_state,
            set_evm_canon_block_hash_and_return_state,
            set_evm_latest_block_hash_and_return_state,
        },
        generate_eth_address::{generate_and_store_eth_address, generate_and_store_evm_address},
        generate_eth_private_key::{generate_and_store_eth_private_key, generate_and_store_evm_private_key},
    },
    eth_database_utils::EthDbUtilsExt,
    validate_eth_block_in_state,
    validate_evm_block_in_state,
    vault_using_cores::VaultUsingCores,
    EthState,
    EthSubmissionMaterial,
};

#[allow(clippy::too_many_arguments)]
fn initialize_eth_core_maybe_with_contract_tx_and_return_state<'a, D: DatabaseInterface>(
    sub_mat: EthSubmissionMaterial,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
    is_for_eth: bool,
    vault_contract: Option<&EthAddress>,
    router_contract: Option<&EthAddress>,
    vault_using_core: Option<&VaultUsingCores>,
    is_native: bool,
) -> Result<EthState<'a, D>> {
    state
        .add_eth_submission_material(sub_mat)
        .and_then(|state| {
            if is_for_eth {
                put_eth_chain_id_in_db_and_return_state(chain_id, state)
            } else {
                put_evm_chain_id_in_db_and_return_state(chain_id, state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                validate_eth_block_in_state(state)
            } else {
                validate_evm_block_in_state(state)
            }
        })
        .and_then(remove_receipts_from_block_in_state)
        .and_then(|state| {
            if is_for_eth {
                add_eth_block_to_db_and_return_state(state)
            } else {
                add_evm_block_to_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state)
            } else {
                put_evm_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_anchor_block_hash_and_return_state(state)
            } else {
                set_evm_anchor_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_latest_block_hash_and_return_state(state)
            } else {
                set_evm_latest_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_canon_block_hash_and_return_state(state)
            } else {
                set_evm_canon_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                generate_and_store_eth_private_key(state)
            } else {
                generate_and_store_evm_private_key(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_tail_block_hash_in_db_and_return_state(state)
            } else {
                put_evm_tail_block_hash_in_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_gas_price_in_db_and_return_state(gas_price, state)
            } else {
                put_evm_gas_price_in_db_and_return_state(gas_price, state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_account_nonce_in_db_and_return_state(state, 0)
            } else {
                put_evm_account_nonce_in_db_and_return_state(state, 0)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_any_sender_nonce_in_db_and_return_state(state)
            } else {
                put_evm_any_sender_nonce_in_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                generate_and_store_eth_address(state)
            } else {
                generate_and_store_evm_address(state)
            }
        })
        .and_then(|state| {
            if is_native {
                CoreType::initialize_native_core(state.eth_db_utils.get_db())?
            } else {
                CoreType::initialize_host_core(state.eth_db_utils.get_db())?
            };
            Ok(state)
        })
        .and_then(|state| match router_contract {
            Some(address) => {
                state
                    .eth_db_utils
                    .put_eth_router_smart_contract_address_in_db(address)?;
                Ok(state)
            },
            None => Ok(state),
        })
        .and_then(|state| match vault_contract {
            None => Ok(state),
            Some(address) => match vault_using_core {
                None => Err("Passed a vault address to the ETH initter but no vault using core type!".into()),
                Some(core_type) => {
                    core_type.put_vault_contract_in_db(&state.eth_db_utils, address)?;
                    Ok(state)
                },
            },
        })
}

pub fn init_v3_host_core<D: DatabaseInterface>(
    db: &D,
    sub_mat: EthSubmissionMaterial,
    chain_id: &EthChainId,
    gas_price: u64,
    confs: u64,
) -> Result<()> {
    initialize_eth_core_maybe_with_contract_tx_and_return_state(
        sub_mat,
        chain_id,
        gas_price,
        confs,
        EthState::init(db),
        false,
        None,
        None,
        None,
        false,
    )
    .and(Ok(()))
}

pub fn init_v3_native_core<D: DatabaseInterface>(
    db: &D,
    sub_mat: EthSubmissionMaterial,
    chain_id: &EthChainId,
    gas_price: u64,
    confs: u64,
    vault: &EthAddress,
    vault_using_core: &VaultUsingCores,
) -> Result<()> {
    initialize_eth_core_maybe_with_contract_tx_and_return_state(
        sub_mat,
        chain_id,
        gas_price,
        confs,
        EthState::init(db),
        true,
        Some(vault),
        Some(&EthAddress::zero()), // NOTE: v3 sentinels do not use a router
        Some(vault_using_core),
        true,
    )
    .and(Ok(()))
}

pub fn initialize_eth_core_with_no_contract_tx<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
    is_native: bool,
) -> Result<EthState<'a, D>> {
    info!("✔ Initializing ETH core with NO contract tx...");
    initialize_eth_core_maybe_with_contract_tx_and_return_state(
        EthSubmissionMaterial::from_str(block_json)?,
        chain_id,
        gas_price,
        canon_to_tip_length,
        state,
        true,
        None,
        None,
        None,
        is_native,
    )
}

pub fn initialize_evm_core_with_no_contract_tx<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
    is_native: bool,
) -> Result<EthState<'a, D>> {
    info!("✔ Initializing EVM core with NO contract tx...");
    initialize_eth_core_maybe_with_contract_tx_and_return_state(
        EthSubmissionMaterial::from_str(block_json)?,
        chain_id,
        gas_price,
        canon_to_tip_length,
        state,
        false,
        None,
        None,
        None,
        is_native,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_eth_core_with_vault_and_router_contracts_and_return_state<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
    vault_contract: &EthAddress,
    router_contract: &EthAddress,
    vault_using_core: &VaultUsingCores,
    is_native: bool,
) -> Result<EthState<'a, D>> {
    info!("✔ Initializing core with vault & router contract...");
    initialize_eth_core_maybe_with_contract_tx_and_return_state(
        EthSubmissionMaterial::from_str(block_json)?,
        chain_id,
        gas_price,
        canon_to_tip_length,
        state,
        true,
        Some(vault_contract),
        Some(router_contract),
        Some(vault_using_core),
        is_native,
    )
}

pub fn initialize_eth_core_with_router_contract_and_return_state<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
    router_contract: &EthAddress,
    is_native: bool,
) -> Result<EthState<'a, D>> {
    info!("✔ Initializing core with vault & router contract...");
    initialize_eth_core_maybe_with_contract_tx_and_return_state(
        EthSubmissionMaterial::from_str(block_json)?,
        chain_id,
        gas_price,
        canon_to_tip_length,
        state,
        true,
        None,
        Some(router_contract),
        None,
        is_native,
    )
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{
        eth_database_utils::EthDbUtilsExt,
        eth_utils::convert_hex_to_eth_address,
        test_utils::get_sample_eth_init_block_string,
    };

    #[test]
    fn should_initialize_eth_core_with_vault_and_router_contracts() {
        let db = get_test_database();
        let state = EthState::init(&db);
        let block_json = get_sample_eth_init_block_string();
        let chain_id = EthChainId::Rinkeby;
        let gas_price = 20000000000;
        let confs = 0;
        let is_native = false;
        let vault_address = convert_hex_to_eth_address("0x866e3fc7043efb8ff3a994f7d59f53fe045d4d7a").unwrap();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6").unwrap();
        let result = initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &block_json,
            &chain_id,
            gas_price,
            confs,
            state,
            &vault_address,
            &router_address,
            &VaultUsingCores::Erc20OnEvm,
            is_native,
        );
        // NOTE: We can't assert the actual output since the private key generation is
        // non-deterministic.
        assert!(result.is_ok());
        let updated_state = result.unwrap();
        // NOTE: But we CAN assert various fields are set correctly...
        assert_eq!(
            updated_state
                .eth_db_utils
                .get_eth_router_smart_contract_address_from_db()
                .unwrap(),
            router_address
        );
        assert_eq!(
            updated_state
                .eth_db_utils
                .get_eth_router_smart_contract_address_from_db()
                .unwrap(),
            router_address
        );
        assert_eq!(
            updated_state
                .eth_db_utils
                .get_erc20_on_evm_smart_contract_address_from_db()
                .unwrap(),
            vault_address
        );
        // TODO assert more things!
    }
}
