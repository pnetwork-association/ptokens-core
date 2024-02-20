use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result, AppError};
use common_eth::{
    check_for_parent_of_eth_block_in_state,
    maybe_add_eth_block_and_receipts_to_db_and_return_state,
    maybe_increment_evm_account_nonce_and_return_eth_state,
    maybe_remove_old_eth_tail_block_and_return_state,
    maybe_remove_receipts_from_eth_canon_block_and_return_state,
    maybe_update_eth_canon_block_hash_and_return_state,
    maybe_update_eth_linker_hash_and_return_state,
    maybe_update_eth_tail_block_hash_and_return_state,
    maybe_update_latest_eth_block_hash_and_return_state,
    parse_eth_submission_material_json_and_put_in_state,
    validate_eth_block_in_state,
    validate_receipts_in_state,
    EthState,
    EthSubmissionMaterialJson,
    EthSubmissionMaterialJsons,
};

use crate::int::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_vault_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
    },
    filter_submission_material::filter_submission_material_for_peg_in_events_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    filter_zero_value_tx_infos::filter_out_zero_value_evm_tx_infos_from_state,
    get_int_output_json::{get_int_output_json, IntOutput, IntOutputs},
    parse_tx_infos::maybe_parse_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_evm_txs_and_add_to_eth_state,
};

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_eth_block_in_state)
        .and_then(|state| state.get_eth_evm_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_evm_tx_infos_from_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_sign_evm_txs_and_add_to_eth_state)
        .and_then(maybe_increment_evm_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(get_int_output_json)
}

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_int_block_to_core<D: DatabaseInterface>(db: &D, block: &str) -> Result<String> {
    info!("✔ Submitting INT block to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJson::from_str(block))
        .and_then(|json| submit_int_block(db, &json))
        .and_then(|output| {
            db.end_transaction()?;
            Ok(output.to_string())
        })
}

/// # Submit INT Blocks to Core
///
/// Submit multiple INT blocks to the core. See `submit_evm_block_to_core` for more information.
pub fn submit_int_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("batch submitting INT blocks to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJsons::from_str(blocks))
        .and_then(|jsons| {
            let mut outputs = vec![];

            for json in jsons.iter() {
                match submit_int_block(db, json) {
                    Ok(o) => {
                        outputs.push(o);
                        continue;
                    },
                    Err(AppError::BlockAlreadyInDbError(e)) => {
                        warn!("block already in db error: {e}");
                        info!("moving on to next block in batch!");
                        continue;
                    },
                    Err(e) => return Err(e),
                }
            }

            Ok(outputs)
        })
        .map(IntOutputs::new)
        .and_then(|outputs| {
            db.end_transaction()?;
            Ok(outputs.to_output().to_string())
        })
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use common::{dictionaries::eth_evm::EthEvmTokenDictionary, test_utils::get_test_database};
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        initialize_evm_core_with_no_contract_tx,
        EthDbUtilsExt,
        EthPrivateKey,
        EvmDbUtils,
        VaultUsingCores,
    };
    use serde_json::json;

    use super::*;
    use crate::{
        int::get_int_output_json::IntOutput,
        test_utils::{
            get_sample_evm_init_block_json_string,
            get_sample_int_init_block_json_string,
            get_sample_router_address,
            get_sample_token_dictionary_entry,
            get_sample_vault_address,
        },
    };

    #[test]
    fn should_submit_int_block_successfully() {
        let db = get_test_database();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_int_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEvm,
            true, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Initialize the EVM side of the core...
        initialize_evm_core_with_no_contract_tx(
            &get_sample_evm_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("0x969c70bccf47406e6d27ec91a12e66aedc7ef23e").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("f39d9bfba0555500b8b2c89cc46e90ae75fa80c23752ebae1ff31e3123d459dd").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        // NOTE: Set the nonce to match that used during the test...
        let evm_nonce = 1;
        db_utils.put_eth_account_nonce_in_db(evm_nonce).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key);
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), evm_nonce);
        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();
        let submission_string = read_to_string("src/test_utils/peg-in-block-1.json").unwrap();
        // NOTE: Finally, submit the block containing the peg in....
        let output = submit_int_block_to_core(&db, &submission_string).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 11544952,
            "evm_signed_transactions": [{
                "_id": format!("pint-on-evm-evm-{}", evm_nonce),
                "broadcast": false,
                "evm_tx_hash": "0x41e75a3cf790fec5123ab3fdb46c38e76b0f6af637b99f8c955437862f43aa68",
                "evm_tx_amount": "1336",
                "evm_tx_recipient": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "witnessed_timestamp": 1638896947,
                "host_token_address": "0xdd9f905a34a6c507c7d68384985905cf5eb032e9",
                "originating_tx_hash": "0x41bda64700fcd700e2c5ec7015da9b224f6c55e4859cb18ea164f2f826bede31",
                "originating_address": "0x0e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6",
                "native_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "evm_signed_tx": "f902ab018504a817c8008306ddd094dd9f905a34a6c507c7d68384985905cf5eb032e980b90244dcdc7dd0000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000053800000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000018002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100ffffffff000000000000000000000000000000000000000000000000000000000000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003c0ffee00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa06955916b5bc93975986b259956e2011f954187984c52775db24bcd4bc47fb70aa067aff07dd13eb78b2e1f6e9e200074c44111062adb702e155bbd45404d980fc4",
                "any_sender_nonce": null,
                "evm_account_nonce": evm_nonce,
                "evm_latest_block_number": 11571205,
                "broadcast_tx_hash": null,
                "broadcast_timestamp": null,
                "any_sender_tx": null,
                "destination_chain_id": "0x0069c322",
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_batch_submit_int_blocks_successfully_even_if_one_already_in_db() {
        let db = get_test_database();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let confirmations = 0;
        let gas_price = 20_000_000_000;

        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_int_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEvm,
            true, // NOTE: is_native
        )
        .unwrap();

        // NOTE: Initialize the EVM side of the core...
        initialize_evm_core_with_no_contract_tx(
            &get_sample_evm_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
        )
        .unwrap();

        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();

        let submission_string = read_to_string("src/test_utils/peg-in-block-1.json").unwrap();
        let block_num = 11544952;

        // NOTE: This totally normal submission should succeed
        submit_int_block_to_core(&db, &submission_string).unwrap();

        // NOTE: However it will fail a second time due to the block already being in the db...
        match submit_int_block_to_core(&db, &submission_string) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(AppError::BlockAlreadyInDbError(e)) => assert_eq!(e.block_num, block_num),
            Err(e) => panic!("wrong error received: {e}"),
        };

        // NOTE: However if the same block forms part of a _batch_ of blocks, the
        // `BlockAlreadyInDbError` should be swallowed, and thus no errors.
        let batch = format!("[{submission_string},{submission_string},{submission_string}]");
        let result = submit_int_blocks_to_core(&db, &batch);
        assert!(result.is_ok());
    }
}
