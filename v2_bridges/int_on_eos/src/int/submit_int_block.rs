use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result, AppError};
use common_eth::{
    check_for_parent_of_eth_block_in_state,
    maybe_add_eth_block_and_receipts_to_db_and_return_state,
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
    divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
    filter_out_zero_tx_infos::filter_out_zero_value_eos_tx_infos_from_state,
    filter_submission_material::filter_submission_material_for_relevant_receipts_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    get_output_json::{get_output_json, IntOutput, IntOutputs},
    maybe_increment_eos_account_nonce_and_return_state,
    parse_tx_info::maybe_parse_eos_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_eos_txs_and_add_to_eth_state,
};

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_eth_block_in_state)
        .and_then(|state| state.get_eos_eth_token_dictionary_from_db_and_add_to_state())
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_relevant_receipts_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_eos_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_eos_tx_infos_from_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(get_output_json)
}

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the INT
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
/// Submit multiple INT blocks to the core.
pub fn submit_int_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("✔ Batch submitting INT blocks to core...");
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
    use std::str::FromStr;

    use common::test_utils::get_test_database;
    use common_chain_ids::EthChainId;
    use common_eos::{initialize_eos_core_inner, EosPrivateKey};
    use common_eth::{
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        EthDbUtils,
        EthDbUtilsExt,
        EthState as IntState,
        VaultUsingCores,
    };
    use serde_json::json;

    use super::*;
    use crate::{
        int::get_output_json::IntOutput,
        test_utils::{
            get_contiguous_int_block_json_strs,
            get_sample_dictionary_1,
            get_sample_eos_init_block_1,
            get_sample_eos_private_key,
            get_sample_int_address,
            get_sample_int_private_key,
            get_sample_router_address,
            get_sample_vault_address,
        },
    };

    #[test]
    fn should_submit_int_block() {
        let db = get_test_database();
        let vault_address = get_sample_vault_address();
        let router_address = get_sample_router_address();

        // NOTE: Initialize the EOS core...
        let eos_chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
        let maybe_eos_account_name = None;
        let maybe_eos_token_symbol = None;
        let eos_init_block = get_sample_eos_init_block_1();
        let is_native = false;
        initialize_eos_core_inner(
            &db,
            eos_chain_id,
            maybe_eos_account_name,
            maybe_eos_token_symbol,
            &eos_init_block,
            is_native,
        )
        .unwrap();

        // NOTE: Overwrite the EOS private key since it's generated randomly above...
        let eos_pk = get_sample_eos_private_key();
        eos_pk.write_to_db(&db).unwrap();
        assert_eq!(EosPrivateKey::get_from_db(&db).unwrap(), eos_pk);

        // NOTE: Initialize the INT side of the core...
        let int_confirmations = 0;
        let int_gas_price = 20_000_000_000;
        let contiguous_int_block_json_strs = get_contiguous_int_block_json_strs();
        let int_init_block = contiguous_int_block_json_strs[0].clone();
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            int_gas_price,
            int_confirmations,
            IntState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEos,
            is_native,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = get_sample_int_address();
        let int_private_key = get_sample_int_private_key();
        let int_db_utils = EthDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Add the token dictionary to the db...
        let dictionary = get_sample_dictionary_1();
        dictionary.save_to_db(&db).unwrap();

        // NOTE: Submit the block with the peg in in it...
        let output =
            IntOutput::from_str(&submit_int_block_to_core(&db, &contiguous_int_block_json_strs[1]).unwrap()).unwrap();
        let expected_output = IntOutput::from_str(&json!({
            "int_latest_block_number":12236006,
            "eos_signed_transactions":[{
                "_id":"pint-on-eos-eos-0",
                "broadcast":false,
                "eos_tx_amount":"0.13370000 IOE",
                "int_tx_amount":"133700000000000000",
                "eos_account_nonce":0,
                "eos_tx_recipient":"ptestpout111",
                "eos_tx_signature":"SIG_K1_KaXyH1xZpAyonUJYrPyqxLMFUhbnLrq7HUH4XmrNN7TH3Gd18WqPt3wgBjMd2hUSmvZsLCuhyqCDEuD1uTjpP5XY5VXuz6",
                "witnessed_timestamp":1651664234,
                "eos_serialized_tx":"5b078d6205253662b554000000000190b1ca98aa49f3740080c92671a531760190b1ca98aa49f37400000000a8ed32324f1042c89ad68c55ae9002cc000000000008494f450000000000350303decaff040069c3222a30783731613434306565396661376639396662396136393765393665633738333962386131363433623800",
                "host_token_address":"intoneostest",
                "originating_tx_hash":"0x0695f2980bc04b8da96406c79468f3e51dbb2a18c4bc0638cff055fa63f309f2",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "eos_latest_block_number":213498746,
                "destination_chain_id":"0x02e7261c",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        }).to_string()).unwrap();
        let expected_num_txs = 1;

        // NOTE: And finally, we assert the output...
        assert_eq!(output.eos_signed_transactions.len(), expected_num_txs);
        let result = output.eos_signed_transactions[0].clone();
        let expected_result = expected_output.eos_signed_transactions[0].clone();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_batch_submit_int_blocks_successfully_even_if_one_already_in_db() {
        let db = get_test_database();
        let vault_address = get_sample_vault_address();
        let router_address = get_sample_router_address();

        // NOTE: Initialize the EOS core...
        let eos_chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
        let maybe_eos_account_name = None;
        let maybe_eos_token_symbol = None;
        let eos_init_block = get_sample_eos_init_block_1();
        let is_native = false;
        initialize_eos_core_inner(
            &db,
            eos_chain_id,
            maybe_eos_account_name,
            maybe_eos_token_symbol,
            &eos_init_block,
            is_native,
        )
        .unwrap();

        // NOTE: Overwrite the EOS private key since it's generated randomly above...
        let eos_pk = get_sample_eos_private_key();
        eos_pk.write_to_db(&db).unwrap();
        assert_eq!(EosPrivateKey::get_from_db(&db).unwrap(), eos_pk);

        // NOTE: Initialize the INT side of the core...
        let int_confirmations = 0;
        let int_gas_price = 20_000_000_000;
        let contiguous_int_block_json_strs = get_contiguous_int_block_json_strs();
        let int_init_block = contiguous_int_block_json_strs[0].clone();
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            int_gas_price,
            int_confirmations,
            IntState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEos,
            is_native,
        )
        .unwrap();

        // NOTE: Add the token dictionary to the db...
        let dictionary = get_sample_dictionary_1();
        dictionary.save_to_db(&db).unwrap();

        let submission_string = contiguous_int_block_json_strs[1].clone();
        let block_num = 12236006;
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
