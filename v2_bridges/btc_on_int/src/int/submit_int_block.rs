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
    filter_receipts_in_state::filter_receipts_for_btc_on_int_redeem_events_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    get_int_output::{get_int_output_json, IntOutput, IntOutputs},
    maybe_increment_btc_account_nonce_and_return_eth_state,
    parse_tx_infos::maybe_parse_btc_on_int_tx_infos_and_add_to_state,
    sign_txs::maybe_sign_btc_txs_and_add_to_state,
};

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_eth_block_in_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_receipts_for_btc_on_int_redeem_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_btc_on_int_tx_infos_and_add_to_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_sign_btc_txs_and_add_to_state)
        .and_then(maybe_increment_btc_account_nonce_and_return_eth_state)
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
    info!("✔ Submitting INT block to common...");
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
/// Submit multiple INT blocks to the common.
pub fn submit_int_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("✔ Batch submitting INT blocks to common...");
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

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use std::str::FromStr;

    use common::test_utils::get_test_database;
    use common_btc::{
        convert_hex_tx_to_btc_transaction,
        get_utxo_nonce_from_db,
        init_btc_core,
        BtcDbUtils,
        BtcPrivateKey,
        BtcState,
    };
    use common_eth::{
        convert_hex_to_eth_address,
        EthDbUtils,
        EthDbUtilsExt,
        EthPrivateKey,
        EthState,
        EthSubmissionMaterial,
    };
    use serde_json::json;

    use super::*;
    use crate::{
        int::{get_int_output::IntOutput, initialize_int_core::init_int_core},
        submit_btc_block_to_core,
        test_utils::{get_sample_btc_submission_material_json_str_n, get_sample_int_submission_material_json_str_n},
    };

    #[test]
    fn should_submit_int_blocks_to_core() {
        // Init the BTC side...
        let btc_pk = "93GJ65qHNjGFHzQVTzEEAdBS7vMxe3XASfWE8RUASSfd3EtfmzP";
        let db = get_test_database();
        let btc_db_utils = BtcDbUtils::new(&db);
        let btc_state = BtcState::init(&db);
        let btc_fee = 15;
        let btc_difficulty = 1;
        let btc_network = "Testnet";
        let btc_canon_to_tip_length = 2;
        let btc_block_0 = get_sample_btc_submission_material_json_str_n(0);
        init_btc_core(
            btc_state,
            &btc_block_0,
            btc_fee,
            btc_difficulty,
            btc_network,
            btc_canon_to_tip_length,
        )
        .unwrap();

        // NOTE: Overwrite the BTC private key fields since they're randomly generated upon init.
        let btc_pk = BtcPrivateKey::from_wif(btc_pk).unwrap();
        let address = btc_pk.to_p2pkh_btc_address();
        btc_db_utils.put_btc_private_key_in_db(&btc_pk).unwrap();
        btc_db_utils.put_btc_address_in_db(&address).unwrap();
        btc_db_utils
            .put_btc_pub_key_slice_in_db(&btc_pk.to_public_key_slice())
            .unwrap();

        // Init the ETH side...
        let eth_block_0 = get_sample_int_submission_material_json_str_n(0);
        let eth_state = EthState::init(&db);
        let eth_chain_id = 3;
        let eth_gas_price = 20_000_000_000;
        let eth_canon_to_tip_length = 2;
        let ptoken_address_hex = "0x0f513aA8d67820787A8FDf285Bfcf967bF8E4B8b";
        let ptoken_address = convert_hex_to_eth_address(ptoken_address_hex).unwrap();
        let router_address_hex = "0x88d19e08cd43bba5761c10c588b2a3d85c75041f";
        let router_address = convert_hex_to_eth_address(router_address_hex).unwrap();
        init_int_core(
            eth_state,
            &eth_block_0,
            eth_chain_id,
            eth_gas_price,
            eth_canon_to_tip_length,
            &ptoken_address,
            &router_address,
        )
        .unwrap();

        // NOTE: Overwrite the ETH private key fields since they're randomly generated upon init.
        let eth_db_utils = EthDbUtils::new(&db);
        let eth_pk_bytes = hex::decode("262e2a3a7fa5ae40ea04584f20b51fc3918b42e7dd89926b9f4e2196c8a032ba").unwrap();
        let eth_pk = EthPrivateKey::from_slice(&eth_pk_bytes).unwrap();
        eth_db_utils.put_eth_private_key_in_db(&eth_pk).unwrap();
        eth_db_utils
            .put_public_eth_address_in_db(&eth_pk.to_public_key().to_address())
            .unwrap();

        // NOTE First we submit enough BTC blocks to have a UTXO to spend...
        let btc_block_1 = get_sample_btc_submission_material_json_str_n(1);
        submit_btc_block_to_core(&db, &btc_block_1).unwrap();
        let btc_block_2 = get_sample_btc_submission_material_json_str_n(2);
        submit_btc_block_to_core(&db, &btc_block_2).unwrap();
        let btc_block_3 = get_sample_btc_submission_material_json_str_n(3);
        submit_btc_block_to_core(&db, &btc_block_3).unwrap();
        let utxo_nonce = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(utxo_nonce, 1);

        // NOTE: Submit first block, this one has a peg in in it.
        let int_block_1 = get_sample_int_submission_material_json_str_n(1);
        let result_1 = submit_int_block_to_core(&db, &int_block_1).unwrap();
        let expected_result_1 = IntOutput::new(
            EthSubmissionMaterial::from_str(&int_block_1)
                .unwrap()
                .block
                .unwrap()
                .number
                .as_u64() as usize,
            vec![],
        );
        assert_eq!(IntOutput::from_str(&result_1).unwrap(), expected_result_1);

        let int_block_2 = get_sample_int_submission_material_json_str_n(2);
        let result_2 = submit_int_block_to_core(&db, &int_block_2).unwrap();
        let expected_result_2 = IntOutput::new(
            EthSubmissionMaterial::from_str(&int_block_2)
                .unwrap()
                .block
                .unwrap()
                .number
                .as_u64() as usize,
            vec![],
        );
        assert_eq!(IntOutput::from_str(&result_2).unwrap(), expected_result_2);

        // NOTE: By now the block with the submission is the canon block, and hence a tx is signed.
        let int_block_3 = get_sample_int_submission_material_json_str_n(3);
        let result = IntOutput::from_str(&submit_int_block_to_core(&db, &int_block_3).unwrap()).unwrap();
        let expected_result = IntOutput::from_str(
            &json!({
                "int_latest_block_number":12000344,
                "btc_signed_transactions":[{
                    "broadcast":false,
                    "btc_tx_amount":1337,
                    "btc_account_nonce":0,
                    "broadcast_tx_hash":null,
                    "_id":"pbtc-on-int-btc-0",
                    "broadcast_timestamp":null,
                    "witnessed_timestamp":1645526102,
                    "btc_latest_block_number":2163205,
                    "host_token_address": "0x0f513aa8d67820787a8fdf285bfcf967bf8e4b8b",
                    "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                    "btc_tx_recipient":"tb1q3m09363jpkrwnc9yepp8eunhunlp59y83k7m7w",
                    "btc_tx_hash":"4e46ab88bb9a3b9d849d3d93ac84d85223c4f61f02068cd3e88d6e9f0bcb97e1",
                    "originating_tx_hash":"0xdc676d1858ebf2a45f8b65ba4a925dfa8012bfeecba21df4b6935e58f4c8fcfa",
                    "btc_signed_tx":"01000000014e635c5f95ba996dc34791193deaceb51218bbea643561f9f2c7b556fe8f77d3010000008f483045022100a7f529b473c52e4a16091580d948d6f4c7d71192b33436701481a68c3a1b31af02202d3726c4403f7883816673188fb6eb73d9ca868faa6b806c63f9450310251ef00145202b69d3bc995c316a478b8b70b82b820505dcd31b80b624a947cceb37882f00c9752103fd539c728597e774040bda920ea7112257422442dcd7d9fc12e04e578e0af91aacffffffff0239050000000000001600148ede58ea320d86e9e0a4c8427cf277e4fe1a148754060000000000001976a914ec8f6a91d8ca2e2875575a17f83f3c2e9238f47188ac00000000",
                    "destination_chain_id": "0x018afeb2",
                }]
            }).to_string()
        ).unwrap();

        // NOTE: Assert the results...
        assert_eq!(result.int_latest_block_number, expected_result.int_latest_block_number);
        assert_eq!(result.btc_signed_transactions.len(), 1);
        let tx_info = result.btc_signed_transactions[0].clone();
        let expected_tx_info = expected_result.btc_signed_transactions[0].clone();
        assert_eq!(tx_info, expected_tx_info);

        // NOTE: Check the tx is decodable...
        assert!(convert_hex_tx_to_btc_transaction(tx_info.btc_signed_tx).is_ok());
    }

    #[test]
    fn should_batch_submit_int_blocks_successfully_even_if_one_already_in_db() {
        // Init the BTC side...
        let btc_pk = "93GJ65qHNjGFHzQVTzEEAdBS7vMxe3XASfWE8RUASSfd3EtfmzP";
        let db = get_test_database();
        let btc_db_utils = BtcDbUtils::new(&db);
        let btc_state = BtcState::init(&db);
        let btc_fee = 15;
        let btc_difficulty = 1;
        let btc_network = "Testnet";
        let btc_canon_to_tip_length = 2;
        let btc_block_0 = get_sample_btc_submission_material_json_str_n(0);
        init_btc_core(
            btc_state,
            &btc_block_0,
            btc_fee,
            btc_difficulty,
            btc_network,
            btc_canon_to_tip_length,
        )
        .unwrap();

        // NOTE: Overwrite the BTC private key fields since they're randomly generated upon init.
        let btc_pk = BtcPrivateKey::from_wif(btc_pk).unwrap();
        let address = btc_pk.to_p2pkh_btc_address();
        btc_db_utils.put_btc_private_key_in_db(&btc_pk).unwrap();
        btc_db_utils.put_btc_address_in_db(&address).unwrap();
        btc_db_utils
            .put_btc_pub_key_slice_in_db(&btc_pk.to_public_key_slice())
            .unwrap();

        // Init the ETH side...
        let eth_block_0 = get_sample_int_submission_material_json_str_n(0);
        let eth_state = EthState::init(&db);
        let eth_chain_id = 3;
        let eth_gas_price = 20_000_000_000;
        let eth_canon_to_tip_length = 2;
        let ptoken_address_hex = "0x0f513aA8d67820787A8FDf285Bfcf967bF8E4B8b";
        let ptoken_address = convert_hex_to_eth_address(ptoken_address_hex).unwrap();
        let router_address_hex = "0x88d19e08cd43bba5761c10c588b2a3d85c75041f";
        let router_address = convert_hex_to_eth_address(router_address_hex).unwrap();
        init_int_core(
            eth_state,
            &eth_block_0,
            eth_chain_id,
            eth_gas_price,
            eth_canon_to_tip_length,
            &ptoken_address,
            &router_address,
        )
        .unwrap();

        // NOTE: Overwrite the ETH private key fields since they're randomly generated upon init.
        let eth_db_utils = EthDbUtils::new(&db);
        let eth_pk_bytes = hex::decode("262e2a3a7fa5ae40ea04584f20b51fc3918b42e7dd89926b9f4e2196c8a032ba").unwrap();
        let eth_pk = EthPrivateKey::from_slice(&eth_pk_bytes).unwrap();
        eth_db_utils.put_eth_private_key_in_db(&eth_pk).unwrap();
        eth_db_utils
            .put_public_eth_address_in_db(&eth_pk.to_public_key().to_address())
            .unwrap();

        // NOTE First we submit enough BTC blocks to have a UTXO to spend...
        let btc_block_1 = get_sample_btc_submission_material_json_str_n(1);
        submit_btc_block_to_core(&db, &btc_block_1).unwrap();
        let btc_block_2 = get_sample_btc_submission_material_json_str_n(2);
        submit_btc_block_to_core(&db, &btc_block_2).unwrap();
        let btc_block_3 = get_sample_btc_submission_material_json_str_n(3);
        submit_btc_block_to_core(&db, &btc_block_3).unwrap();
        let utxo_nonce = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(utxo_nonce, 1);

        // NOTE: Submit first block, this one has a peg in in it.
        let int_block_1 = get_sample_int_submission_material_json_str_n(1);

        let submission_string = int_block_1.clone();
        let block_num = 12000342;

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
