use common::{traits::DatabaseInterface, types::Result, CoreType};
use common_eos::{
    end_eos_db_transaction_and_return_state,
    get_active_schedule_from_db_and_add_to_state,
    get_enabled_protocol_features_and_add_to_state,
    get_processed_global_sequences_and_add_to_state,
    maybe_add_global_sequences_to_processed_list_and_return_state,
    maybe_add_new_eos_schedule_to_db_and_return_state,
    maybe_filter_duplicate_proofs_from_state,
    maybe_filter_out_action_proof_receipt_mismatches_and_return_state,
    maybe_filter_out_invalid_action_receipt_digests,
    maybe_filter_out_proofs_for_wrong_eos_account_name,
    maybe_filter_out_proofs_with_invalid_merkle_proofs,
    maybe_filter_out_proofs_with_wrong_action_mroot,
    maybe_filter_proofs_for_v1_peg_in_actions,
    parse_submission_material_and_add_to_state,
    validate_block_header_signature,
    validate_producer_slot_of_block_in_state,
    EosState,
    Incremerkles,
};

use crate::eos::{
    divert_to_safe_address::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
    },
    filter_txs::{maybe_filter_out_already_processed_tx_ids_from_state, maybe_filter_out_value_too_low_txs_from_state},
    get_eos_output::get_eos_output,
    increment_int_nonce::maybe_increment_int_nonce_in_db_and_return_eos_state,
    parse_tx_info::maybe_parse_eos_on_int_int_tx_infos_and_put_in_state,
    sign_txs::maybe_sign_int_txs_and_add_to_state,
};

/// # Submit EOS Block to Core
///
/// The main submission pipeline. Submitting an EOS block to the enclave will - if the block is
/// valid & the accompanying transaction IDs update the incremerkle correctly - advanced the core's
/// incremerkle accordingly. Any proofs submitted with the block and transaction IDs will then be
/// parsed and if found to pertain to peg outs made in the block in question, an ETH transaction
/// will be signed and returned to the caller.
#[cfg(feature = "spring_1-0")]
pub fn submit_eos_block_to_core<D: DatabaseInterface>(db: &D, block_json: &str) -> Result<String> {
    info!("✔ Submitting EOS block to core...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| parse_submission_material_and_add_to_state(block_json, EosState::init(db)))
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(get_active_schedule_from_db_and_add_to_state)
        .and_then(|state| state.get_eos_eth_token_dictionary_and_add_to_state())
        .and_then(validate_producer_slot_of_block_in_state)
        .and_then(validate_block_header_signature)
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_filter_out_proofs_for_wrong_eos_account_name)
        .and_then(maybe_filter_proofs_for_v1_peg_in_actions)
        .and_then(maybe_parse_eos_on_int_int_tx_infos_and_put_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_filter_out_value_too_low_txs_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(maybe_sign_int_txs_and_add_to_state)
        .and_then(maybe_increment_int_nonce_in_db_and_return_eos_state)
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_output)
}

#[cfg(not(feature = "spring_1-0"))]
pub fn submit_eos_block_to_core<D: DatabaseInterface>(db: &D, block_json: &str) -> Result<String> {
    info!("✔ Submitting EOS block to core...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| parse_submission_material_and_add_to_state(block_json, EosState::init(db)))
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(Incremerkles::get_from_db_and_add_to_state)
        .and_then(Incremerkles::add_block_ids_and_return_state)
        .and_then(get_active_schedule_from_db_and_add_to_state)
        .and_then(|state| state.get_eos_eth_token_dictionary_and_add_to_state())
        .and_then(validate_producer_slot_of_block_in_state)
        .and_then(validate_block_header_signature)
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_filter_out_proofs_for_wrong_eos_account_name)
        .and_then(maybe_filter_proofs_for_v1_peg_in_actions)
        .and_then(maybe_parse_eos_on_int_int_tx_infos_and_put_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_filter_out_value_too_low_txs_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(maybe_sign_int_txs_and_add_to_state)
        .and_then(maybe_increment_int_nonce_in_db_and_return_eos_state)
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_output)
}

// NOTE: The test uses TELOS blocks, whose headers fail validation.
#[cfg(all(test, feature = "non-validating"))]
mod tests {
    use std::str::FromStr;

    use common::test_utils::get_test_database;
    use common_chain_ids::EthChainId;
    use common_eos::{initialize_eos_core_inner, EosPrivateKey, EosSubmissionMaterial, ProcessedGlobalSequences};
    use common_eth::{
        initialize_eth_core_with_router_contract_and_return_state,
        EthDbUtils,
        EthDbUtilsExt,
        EthState as IntState,
    };
    use serde_json::json;

    use super::*;
    use crate::{
        eos::get_eos_output::EosOutput,
        test_utils::{
            get_contiguous_int_block_json_strs,
            get_sample_dictionary,
            get_sample_eos_init_block,
            get_sample_eos_private_key,
            get_sample_eos_submission_material_string,
            get_sample_int_address,
            get_sample_int_private_key,
            get_sample_router_address,
            multi_incremerkle_submission::{
                get_incremekle_update_block,
                get_init_block,
                get_sample_dictionary as get_sample_dictionary_for_incremerkle_test,
                get_submission_block,
            },
        },
    };

    #[cfg(feature = "spring_1-0")]
    mod spring_1_0_blocks {
        use super::*;

        #[test]
        fn should_submit_eos_block() {
            let db = get_test_database();
            let router_address = get_sample_router_address();

            // NOTE: Initialize the EOS core...
            let eos_chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
            let maybe_eos_account_name = Some("intoneostest");
            let maybe_eos_token_symbol = None;
            let eos_init_block = get_sample_eos_init_block();
            initialize_eos_core_inner(
                &db,
                eos_chain_id,
                maybe_eos_account_name,
                maybe_eos_token_symbol,
                &eos_init_block,
                true,
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
            initialize_eth_core_with_router_contract_and_return_state(
                &int_init_block,
                &EthChainId::Ropsten,
                int_gas_price,
                int_confirmations,
                IntState::init(&db),
                &router_address,
                false,
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
            let dictionary = get_sample_dictionary();
            dictionary.save_to_db(&db).unwrap();

            // NOTE: Assert that there are no processed global sequences in the db...
            let processed_glob_sequences_before = ProcessedGlobalSequences::get_from_db(&db).unwrap();
            assert!(processed_glob_sequences_before.is_empty());

            // NOTE: Submit the block with the peg in in it...
            let output = EosOutput::from_str(
                &submit_eos_block_to_core(&db, &get_sample_eos_submission_material_string()).unwrap(),
            )
            .unwrap();

            let expected_output = EosOutput::from_str(&json!({
            "eos_latest_block_number":222279899,
            "int_signed_transactions":[{
                "_id":"peos-on-int-int-0",
                "broadcast":false,
                "int_tx_hash":"0x02f0f3a27b18fb9fd28a16b114566a722e5dad549ff5e02cb73e0bfe90f7eb0b",
                "int_tx_amount":"100000000000000000",
                "int_account_nonce":0,
                "int_tx_recipient":"0x988e8C89cca8f54f144D270BCFB02C4584F005E6",
                "witnessed_timestamp":1656079533,
                "host_token_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "originating_tx_hash": "65c51028ac71cf1fd52e75e6839cac53186698dea7b57c07f9dac574ee1fff76",
                "originating_address":"ptestpout111",
                "native_token_address":"ptestpout123",
                "destination_chain_id":"0x005fe7f9",
                "int_signed_tx":"f9034b808504a817c8008306ddd094a83446f219baec0b6fd6b3031c5a49a54543045b80b902e4dcdc7dd0000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c102000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000000022003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100028c7109000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140005fe7f900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000004005fe7f90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000081042c89ad68c55ae000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307839383865386338396363613866353466313434643237306263666230326334353834663030356536000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0050c1852ad4a56a38993fd0ba4a77dd1a6bcf9a12306f39f02d20b9b3be70275a0280845dcb8628ed970296b0067ba7b6e50e902414346c7e92f7191bc4b94c5f8",
                "int_latest_block_number":11618226,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
        }]}).to_string()).unwrap();

            // NOTE: And finally, we assert the output...
            let expected_num_txs = 1;
            assert_eq!(output.int_signed_transactions.len(), expected_num_txs);
            let result = output.int_signed_transactions[0].clone();
            let expected_result = expected_output.int_signed_transactions[0].clone();
            assert_eq!(result._id, expected_result._id);
            assert_eq!(result.broadcast, expected_result.broadcast);
            assert_eq!(result.int_tx_hash, expected_result.int_tx_hash);
            assert_eq!(result.int_tx_amount, expected_result.int_tx_amount);
            assert_eq!(result.int_tx_amount, expected_result.int_tx_amount);
            assert_eq!(result.int_account_nonce, expected_result.int_account_nonce);
            assert_eq!(result.int_tx_recipient, expected_result.int_tx_recipient);
            assert_eq!(result.host_token_address, expected_result.host_token_address);
            assert_eq!(result.originating_tx_hash, expected_result.originating_tx_hash);
            assert_eq!(result.originating_address, expected_result.originating_address);
            assert_eq!(result.native_token_address, expected_result.native_token_address);
            assert_eq!(result.int_signed_tx, expected_result.int_signed_tx);
            assert_eq!(result.int_latest_block_number, expected_result.int_latest_block_number);
            assert_eq!(result.broadcast_tx_hash, expected_result.broadcast_tx_hash);
            assert_eq!(result.broadcast_timestamp, expected_result.broadcast_timestamp);
            assert_eq!(result.destination_chain_id, expected_result.destination_chain_id);
            // NOTE: We don't assert the timestamp since it's not deterministic.

            // NOTE: Assert that we've processed the expected action...
            let processed_glob_sequences_after = ProcessedGlobalSequences::get_from_db(&db).unwrap();
            assert!(processed_glob_sequences_after.contains(&9854285413));
        }
    }

    #[cfg(not(feature = "spring_1-0"))]
    mod legacy_blocks {
        use super::*;

        #[test]
        fn should_submit_eos_block() {
            let db = get_test_database();
            let router_address = get_sample_router_address();

            // NOTE: Initialize the EOS core...
            let eos_chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
            let maybe_eos_account_name = Some("intoneostest");
            let maybe_eos_token_symbol = None;
            let eos_init_block = get_sample_eos_init_block();
            initialize_eos_core_inner(
                &db,
                eos_chain_id,
                maybe_eos_account_name,
                maybe_eos_token_symbol,
                &eos_init_block,
                true,
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
            initialize_eth_core_with_router_contract_and_return_state(
                &int_init_block,
                &EthChainId::Ropsten,
                int_gas_price,
                int_confirmations,
                IntState::init(&db),
                &router_address,
                false,
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
            let dictionary = get_sample_dictionary();
            dictionary.save_to_db(&db).unwrap();

            // NOTE: Assert that there are no processed global sequences in the db...
            let processed_glob_sequences_before = ProcessedGlobalSequences::get_from_db(&db).unwrap();
            assert!(processed_glob_sequences_before.is_empty());

            let mut incremerkles = Incremerkles::get_from_db(&common_eos::EosDbUtils::new(&db)).unwrap();
            assert_eq!(incremerkles.len(), 1);
            assert_eq!(incremerkles.latest_block_num(), 222275383);

            // NOTE: Submit the block with the peg in in it...
            let output = EosOutput::from_str(
                &submit_eos_block_to_core(&db, &get_sample_eos_submission_material_string()).unwrap(),
            )
            .unwrap();

            // NOTE We should now have two incremerkles stored in the db
            incremerkles = Incremerkles::get_from_db(&common_eos::EosDbUtils::new(&db)).unwrap();
            assert_eq!(incremerkles.len(), 2);
            assert_eq!(incremerkles.latest_block_num(), 222279899);

            let expected_output = EosOutput::from_str(&json!({
            "eos_latest_block_number":222279899,
            "int_signed_transactions":[{
                "_id":"peos-on-int-int-0",
                "broadcast":false,
                "int_tx_hash":"0x02f0f3a27b18fb9fd28a16b114566a722e5dad549ff5e02cb73e0bfe90f7eb0b",
                "int_tx_amount":"100000000000000000",
                "int_account_nonce":0,
                "int_tx_recipient":"0x988e8C89cca8f54f144D270BCFB02C4584F005E6",
                "witnessed_timestamp":1656079533,
                "host_token_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "originating_tx_hash": "65c51028ac71cf1fd52e75e6839cac53186698dea7b57c07f9dac574ee1fff76",
                "originating_address":"ptestpout111",
                "native_token_address":"ptestpout123",
                "destination_chain_id":"0x005fe7f9",
                "int_signed_tx":"f9034b808504a817c8008306ddd094a83446f219baec0b6fd6b3031c5a49a54543045b80b902e4dcdc7dd0000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c102000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000000022003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100028c7109000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000140005fe7f900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000004005fe7f90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000081042c89ad68c55ae000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307839383865386338396363613866353466313434643237306263666230326334353834663030356536000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0050c1852ad4a56a38993fd0ba4a77dd1a6bcf9a12306f39f02d20b9b3be70275a0280845dcb8628ed970296b0067ba7b6e50e902414346c7e92f7191bc4b94c5f8",
                "int_latest_block_number":11618226,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
        }]}).to_string()).unwrap();

            // NOTE: And finally, we assert the output...
            let expected_num_txs = 1;
            assert_eq!(output.int_signed_transactions.len(), expected_num_txs);
            let result = output.int_signed_transactions[0].clone();
            let expected_result = expected_output.int_signed_transactions[0].clone();
            assert_eq!(result._id, expected_result._id);
            assert_eq!(result.broadcast, expected_result.broadcast);
            assert_eq!(result.int_tx_hash, expected_result.int_tx_hash);
            assert_eq!(result.int_tx_amount, expected_result.int_tx_amount);
            assert_eq!(result.int_tx_amount, expected_result.int_tx_amount);
            assert_eq!(result.int_account_nonce, expected_result.int_account_nonce);
            assert_eq!(result.int_tx_recipient, expected_result.int_tx_recipient);
            assert_eq!(result.host_token_address, expected_result.host_token_address);
            assert_eq!(result.originating_tx_hash, expected_result.originating_tx_hash);
            assert_eq!(result.originating_address, expected_result.originating_address);
            assert_eq!(result.native_token_address, expected_result.native_token_address);
            assert_eq!(result.int_signed_tx, expected_result.int_signed_tx);
            assert_eq!(result.int_latest_block_number, expected_result.int_latest_block_number);
            assert_eq!(result.broadcast_tx_hash, expected_result.broadcast_tx_hash);
            assert_eq!(result.broadcast_timestamp, expected_result.broadcast_timestamp);
            assert_eq!(result.destination_chain_id, expected_result.destination_chain_id);
            // NOTE: We don't assert the timestamp since it's not deterministic.

            // NOTE: Assert that we've processed the expected action...
            let processed_glob_sequences_after = ProcessedGlobalSequences::get_from_db(&db).unwrap();
            assert!(processed_glob_sequences_after.contains(&9854285413));
        }

        #[test]
        fn should_submit_eos_material_with_proof_tied_to_block_behind_chain_tip() {
            let db = get_test_database();
            let router_address = get_sample_router_address();

            // NOTE: Initialize the EOS mainnet core...
            let eos_chain_id = "aca376f206b8fc25a6ed44dbdc66547c36c6c33e3a119ffbeaef943642f0e906";
            let maybe_eos_account_name = Some("x.ptokens");
            let maybe_eos_token_symbol = None;
            let eos_init_block = get_init_block();
            let eos_init_block_num = 351734756;
            initialize_eos_core_inner(
                &db,
                eos_chain_id,
                maybe_eos_account_name,
                maybe_eos_token_symbol,
                &eos_init_block,
                true,
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
            initialize_eth_core_with_router_contract_and_return_state(
                &int_init_block,
                &EthChainId::Ropsten,
                int_gas_price,
                int_confirmations,
                IntState::init(&db),
                &router_address,
                false,
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
            let dictionary = get_sample_dictionary_for_incremerkle_test();
            dictionary.save_to_db(&db).unwrap();

            // NOTE: Assert that there are no processed global sequences in the db...
            let processed_glob_sequences_before = ProcessedGlobalSequences::get_from_db(&db).unwrap();
            assert!(processed_glob_sequences_before.is_empty());

            let mut incremerkles = Incremerkles::get_from_db(&common_eos::EosDbUtils::new(&db)).unwrap();
            assert_eq!(incremerkles.len(), 1);
            assert_eq!(incremerkles.latest_block_num(), eos_init_block_num);

            // NOTE: Now lets update the incremerkle...
            let incremerkle_update_block = get_incremekle_update_block();
            let incremerkle_update_block_num = EosSubmissionMaterial::from_str(&incremerkle_update_block)
                .unwrap()
                .block_num;
            submit_eos_block_to_core(&db, &incremerkle_update_block).unwrap();

            incremerkles = Incremerkles::get_from_db(&common_eos::EosDbUtils::new(&db)).unwrap();
            assert_eq!(incremerkles.len(), 2);
            assert_eq!(incremerkles.latest_block_num(), incremerkle_update_block_num);

            let submission_block_json = get_submission_block();
            let submission_block_num = EosSubmissionMaterial::from_str(&submission_block_json)
                .unwrap()
                .block_num;
            assert_eq!(submission_block_num, 351734979);

            // NOTE: Let's assert that the core's latest block number is indeed _past_ the submission
            // material block num
            let latest_block_num = incremerkles.latest_block_num();
            assert_eq!(latest_block_num, 351735116);
            assert!(latest_block_num > submission_block_num);

            // NOTE: Now we can submit the block with the peg in in it...
            let mut output =
                EosOutput::from_str(&submit_eos_block_to_core(&db, &submission_block_json).unwrap()).unwrap();
            // NOTE: Asserting a tx is outputted successfully is sufficient for this test.
            assert_eq!(output.int_signed_transactions.len(), 1);

            // NOTE: If we submit the _same_ material again, we should get no signed transactions since
            // the core has already seen this global sequence.
            output = EosOutput::from_str(&submit_eos_block_to_core(&db, &submission_block_json).unwrap()).unwrap();
            assert_eq!(output.int_signed_transactions.len(), 0);
        }
    }
}
