use common::{traits::DatabaseInterface, types::Result};

use crate::{
    bitcoin_crate_alias::blockdata::{script::Script as BtcScript, transaction::Transaction as BtcTransaction},
    btc_utils::{create_unsigned_utxo_from_tx, get_pay_to_pub_key_hash_script},
    utxo_manager::{BtcUtxoAndValue, BtcUtxosAndValues},
    BtcState,
};

pub fn extract_utxos_from_p2pkh_tx(target_script: &BtcScript, tx: &BtcTransaction) -> BtcUtxosAndValues {
    BtcUtxosAndValues::new(
        tx.output
            .iter()
            .enumerate()
            .filter(|(_, output)| &output.script_pubkey == target_script)
            .map(|(index, output)| {
                BtcUtxoAndValue::new(
                    output.value,
                    &create_unsigned_utxo_from_tx(tx, index as u32),
                    None,
                    None,
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub fn extract_utxos_from_p2pkh_txs(target_script: &BtcScript, txs: &[BtcTransaction]) -> BtcUtxosAndValues {
    info!("✔ Extracting UTXOs from {} `p2pkh` txs...", txs.len());
    BtcUtxosAndValues::new(
        txs.iter()
            .flat_map(|tx| extract_utxos_from_p2pkh_tx(target_script, tx))
            .collect::<Vec<BtcUtxoAndValue>>(),
    )
}

pub fn maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Maybe extracting UTXOs from `p2pkh` txs...");
    state
        .btc_db_utils
        .get_btc_address_from_db()
        .and_then(|btc_address| get_pay_to_pub_key_hash_script(&btc_address))
        .and_then(|target_script| {
            Ok(extract_utxos_from_p2pkh_txs(
                &target_script,
                state.get_p2pkh_deposit_txs()?,
            ))
        })
        .and_then(|utxos| {
            debug!("✔ Extracted UTXOs: {:?}", utxos);
            info!("✔ Extracted {} `p2pkh` UTXOs", utxos.len());
            state.add_utxos_and_values(utxos)
        })
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use super::*;
    use crate::{
        btc_utils::create_unsigned_utxo_from_tx,
        test_utils::{
            get_sample_btc_tx,
            get_sample_btc_utxo,
            get_sample_p2pkh_utxo_and_value,
            get_sample_pay_to_pub_key_hash_script,
            get_sample_testnet_block_and_txs,
            SAMPLE_OUTPUT_INDEX_OF_UTXO,
        },
        utxo_manager::BtcUtxosAndValues,
    };

    #[test]
    fn should_create_unsigned_utxo_from_tx_output() {
        let tx = get_sample_btc_tx();
        let result = create_unsigned_utxo_from_tx(&tx, SAMPLE_OUTPUT_INDEX_OF_UTXO);
        let expected_result = get_sample_btc_utxo();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_extract_utxos_from_relevant_txs() {
        let expected_num_utxos = 1;
        let expected_utxo_and_value = get_sample_p2pkh_utxo_and_value();
        let txs = get_sample_testnet_block_and_txs().unwrap().block.txdata;
        let target_script = get_sample_pay_to_pub_key_hash_script();
        let result = extract_utxos_from_p2pkh_txs(&target_script, &txs);
        assert_eq!(result.len(), expected_num_utxos);
        assert_eq!(result, BtcUtxosAndValues::new(vec![expected_utxo_and_value]));
    }
}
