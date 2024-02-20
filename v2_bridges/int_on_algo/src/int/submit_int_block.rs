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
    filter_submission_material::filter_submission_material_for_peg_in_events_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
    get_int_output_json::{get_int_output_json, IntOutput, IntOutputs},
    increment_algo_account_nonce::maybe_increment_algo_account_nonce_and_return_eth_state,
    parse_tx_infos::maybe_parse_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_algo_txs_and_add_to_state,
};

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_eth_block_in_state)
        .and_then(|state| state.get_evm_algo_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(maybe_sign_algo_txs_and_add_to_state)
        .and_then(maybe_increment_algo_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(get_int_output_json)
}

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the INT
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an ALGO
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

    use common::{
        constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
        dictionaries::evm_algo::EvmAlgoTokenDictionary,
        test_utils::get_test_database,
        utils::get_prefixed_db_key,
    };
    use common_algo::AlgoDbUtils;
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        EthDbUtilsExt,
        EthPrivateKey,
        EvmDbUtils,
        VaultUsingCores,
    };
    use rust_algorand::{AlgorandAddress, AlgorandGenesisId, AlgorandKeys};
    use serde_json::json;
    use serial_test::serial;

    use super::*;
    use crate::{
        int::get_int_output_json::IntOutput,
        maybe_initialize_algo_core,
        test_utils::{
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1,
            get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in,
            get_sample_contiguous_int_submission_json_strings_for_app_id_peg_in,
            get_sample_contiguous_int_submission_json_strings_for_msg_pack_encoded_user_data,
            get_sample_evm_algo_dictionary_entry_1,
            get_sample_router_address,
            get_sample_vault_address,
        },
    };
    #[test]
    #[serial]
    fn should_submit_int_block_with_address_type_peg_in_successfully() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in();
        let int_init_block = int_submission_material[0].clone();
        let int_peg_in_block = int_submission_material[1].clone();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_submission_material[0],
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let int_db_utils = EvmDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address and private key since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        let algo_private_key = AlgorandKeys::from_bytes(
            &hex::decode("4c9a9699eedc1b7f62b679e375c32ed83159d22428892b7f4285dad2f550f558").unwrap(),
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_private_key_key").to_vec(),
            algo_private_key.to_bytes(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);
        assert_eq!(algo_db_utils.get_algo_private_key().unwrap(), algo_private_key);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Finally, submit the block containing the peg in!
        let output = submit_int_block_to_core(&db, &int_peg_in_block).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 12221814,
            "algo_signed_transactions":[{
                "_id":"pint-on-algo-algo-0",
                "broadcast":false,
                "algo_tx_hash":"1soiRIbdHSK7R+++IlplUbcmjDPNe4N4hehK3ySVkig=",
                "algo_signed_tx":"82a3736967c440eae057ba9511a7671a1a1be716cbf27084df6183f236986f18ee9c53079e8d3c5bdb68f73a2b7b186f032a24ee23089d6483890384f0d3ac29df6249d22dc103a374786e8ba461616d74cd0539a461726376c420367b0a14cd9aa425e23421b4e5339b783f20b35dc08b75f7b9dac22cc4c67b18a3666565cd03e8a26676ce013b42d6a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa3677270c420d6ca224486dd1d22bb47efbe225a6551b7268c33cd7b837885e84adf24959228a26c76ce013b46bea46e6f7465c48198c40103c403decaffc404ffffffffc42a307866656466653236313665623336363163623866656432373832663566306363393164353964636163c40403c38e67c43a474b54355858364e3435555633454e4d494f41564637455151594c37375034355846485949504246414a554f4e37524255435150583537325449c400c400a3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a56178666572a478616964ce2a98f05882a3736967c4405c6390e1a5265cd41ae03d8d642aa671cfce856a14563dee061281400f4b00e87316b794de0d2594b0219b1900772d25183b394adf3bb4bca970cf04f5c59105a374786e8ba46170616192c4056973737565c42032a7dbdfcde7695d91ac438152fc908617ffbf9db94f843c250268e6fe21a0a0a46170617391ce2a98f058a46170617491c42032a7dbdfcde7695d91ac438152fc908617ffbf9db94f843c250268e6fe21a0a0a461706964cd0539a3666565cd03e8a26676ce013b42d6a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa3677270c420d6ca224486dd1d22bb47efbe225a6551b7268c33cd7b837885e84adf24959228a26c76ce013b46bea3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a46170706c",
                "algo_tx_amount":"1337",
                "algo_account_nonce":0,
                "witnessed_timestamp":1650896969,
                "algo_tx_recipient":"GKT5XX6N45UV3ENMIOAVF7EQQYL77P45XFHYIPBFAJUON7RBUCQPX572TI",
                "host_token_address":"714666072",
                "originating_tx_hash":"0xb81f5564195f022f9812d5dfe80052afdfaf8cc86243a98b0dbdd887ef97bda7",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "destination_chain_id":"0x03c38e67",
                "algo_latest_block_number":20642396,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[serial]
    fn should_submit_int_block_with_app_id_type_peg_in_successfully() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_app_id_peg_in();
        let int_init_block = int_submission_material[0].clone();
        let int_peg_in_block = int_submission_material[1].clone();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_submission_material[0],
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let int_db_utils = EvmDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address and private key since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        let algo_private_key = AlgorandKeys::from_bytes(
            &hex::decode("4c9a9699eedc1b7f62b679e375c32ed83159d22428892b7f4285dad2f550f558").unwrap(),
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_private_key_key").to_vec(),
            algo_private_key.to_bytes(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);
        assert_eq!(algo_db_utils.get_algo_private_key().unwrap(), algo_private_key);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Finally, submit the block containing the peg in!
        let output = submit_int_block_to_core(&db, &int_peg_in_block).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 12322611,
            "algo_signed_transactions":[{
                "_id":"pint-on-algo-algo-0",
                "broadcast":false,
                "algo_tx_hash":"GXOei7d98VnC4K0rzm57Zt0LpNdOChU4oTZfHCyzw5A=",
                "algo_signed_tx":"82a3736967c44061ed27c940f02da5a0fd5dc42c2fb15a161cd12dacb80111bdb8af944014b81ffddd43b5c68a400fb65c763d9fc740cae504c39d84cc9418bd5f321f7e56d606a374786e8ba461616d74ce00020b0ca461726376c420367b0a14cd9aa425e23421b4e5339b783f20b35dc08b75f7b9dac22cc4c67b18a3666565cd03e8a26676ce013b42d7a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa3677270c42019739e8bb77df159c2e0ad2bce6e7b66dd0ba4d74e0a1538a1365f1c2cb3c390a26c76ce013b46bfa46e6f7465c45098c40103c403decaffc404ffffffffc42a307866656466653236313665623336363163623866656432373832663566306363393164353964636163c40403c38e67c409373631343037313635c400c400a3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a56178666572a478616964ce2a98f05882a3736967c440363e67b99db85c44ae89e8b746fc605360af45cb3ce68a185cdfc76658993b4a19388dec45f64dffdd0725f33f6deea24c6a26b3beb26197b5b15ffaffee3f03a374786e8ca46170616192c4056973737565c408000000002d6226bda46170617391ce2a98f058a46170617491c420bd605c215b7b89088513be5077f9cc4356a3f70eaccb143f82c4a98813978661a46170666191ce2d6226bda461706964cd0539a3666565cd03e8a26676ce013b42d7a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa3677270c42019739e8bb77df159c2e0ad2bce6e7b66dd0ba4d74e0a1538a1365f1c2cb3c390a26c76ce013b46bfa3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a46170706c",
                "algo_tx_amount":"133900",
                "algo_account_nonce":0,
                "witnessed_timestamp":1650896969,
                "algo_tx_recipient":"761407165",
                "host_token_address":"714666072",
                "originating_tx_hash":"0x9281fcf8992bbfb2d47bde2236b62b677fc5f388a2e0cc0b9c82fe663c441ee1",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "destination_chain_id":"0x03c38e67",
                "algo_latest_block_number":20642396,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[serial]
    fn should_submit_block_with_app_call_tx_and_msg_pack_encoded_user_data_correctly() {
        let db = get_test_database();
        let int_submission_material =
            get_sample_contiguous_int_submission_json_strings_for_msg_pack_encoded_user_data();
        let int_init_block = int_submission_material[0].clone();
        let int_peg_in_block = int_submission_material[1].clone();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_submission_material[0],
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let int_db_utils = EvmDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address and private key since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        let algo_private_key = AlgorandKeys::from_bytes(
            &hex::decode("4c9a9699eedc1b7f62b679e375c32ed83159d22428892b7f4285dad2f550f558").unwrap(),
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_private_key_key").to_vec(),
            algo_private_key.to_bytes(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);
        assert_eq!(algo_db_utils.get_algo_private_key().unwrap(), algo_private_key);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Finally, submit the block containing the peg in!
        let output = submit_int_block_to_core(&db, &int_peg_in_block).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 12342414,
            "algo_signed_transactions":[{
                "_id":"pint-on-algo-algo-0",
                "broadcast":false,
                "algo_tx_hash":"u78V+KugL2SGFrqBMJv2PUpfpe4gJlDkjXfQn0Z5zyw=",
                "algo_signed_tx":"82a3736967c4408fd831e7199f254eb9b1f52b3002250c1a73984165f852e03ed5f6048883bf6deb969c4c1aa51d07e11f73ef61ee366afbc68fdec8d555c40b2fb9567290250aa374786e8ba461616d74ce000207eca461726376c420367b0a14cd9aa425e23421b4e5339b783f20b35dc08b75f7b9dac22cc4c67b18a3666565cd03e8a26676ce013b42d7a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa3677270c420bbbf15f8aba02f648616ba81309bf63d4a5fa5ee202650e48d77d09f4679cf2ca26c76ce013b46bfa46e6f7465c49898c40103c44b95c4208cf5862026483a9b12b349d126811ed1cebcb759b63556130be48ce3e062fc5bc40800000000248759b7c40800000000000003e8c40800000000248759b7c4080000000000000001c404ffffffffc42a307866656466653236313665623336363163623866656432373832663566306363393164353964636163c40403c38e67c409373631343037313635c400c400a3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a56178666572a478616964ce2a98f05882a3736967c4403b029d9f4568c18eff88d765bd516a51b66da4f723cd6f91dc31bcc7e342db4e44febd50ce2b6d7ae4a7680aaf9d7e322ed1959ecfef5807514b22a363f7c90ba374786e8ca46170616192c4056973737565c408000000002d6226bda46170617393ce2a98f058ce248759b7ce248759b7a46170617492c420bd605c215b7b89088513be5077f9cc4356a3f70eaccb143f82c4a98813978661c4208cf5862026483a9b12b349d126811ed1cebcb759b63556130be48ce3e062fc5ba46170666191ce2d6226bda461706964cd0539a3666565cd03e8a26676ce013b42d7a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa3677270c420bbbf15f8aba02f648616ba81309bf63d4a5fa5ee202650e48d77d09f4679cf2ca26c76ce013b46bfa3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a46170706c",
                "algo_tx_amount":"133100",
                "algo_account_nonce":0,
                "witnessed_timestamp":1650896969,
                "algo_tx_recipient":"761407165",
                "host_token_address":"714666072",
                "originating_tx_hash":"0xdd8762861c32a86130279209c068a1f4b2ad44a6ec82bd5d3c4e7a8543808fe8",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "destination_chain_id":"0x03c38e67",
                "algo_latest_block_number":20642396,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&output).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[serial]
    fn should_batch_submit_int_blocks_successfully_even_if_one_already_in_db() {
        let db = get_test_database();
        let int_submission_material =
            get_sample_contiguous_int_submission_json_strings_for_msg_pack_encoded_user_data();
        let int_init_block = int_submission_material[0].clone();
        let int_peg_in_block = int_submission_material[1].clone();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_submission_material[0],
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let int_db_utils = EvmDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address and private key since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        let algo_private_key = AlgorandKeys::from_bytes(
            &hex::decode("4c9a9699eedc1b7f62b679e375c32ed83159d22428892b7f4285dad2f550f558").unwrap(),
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_private_key_key").to_vec(),
            algo_private_key.to_bytes(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);
        assert_eq!(algo_db_utils.get_algo_private_key().unwrap(), algo_private_key);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        let submission_string = int_peg_in_block.clone();
        let block_num = 12342414;

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
