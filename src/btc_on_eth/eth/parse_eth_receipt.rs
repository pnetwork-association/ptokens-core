use ethereum_types::{
    U256,
    H160,
    Address,
};
use crate::{
    types::Result,
    btc_on_eth::{
        utils::{
            convert_hex_to_h256,
            convert_hex_to_address,
            convert_json_value_to_string,
        },
        eth::{
            eth_types::{
                EthReceipt,
                EthReceipts,
                EthReceiptJson,
            },
            get_eth_log::{
                get_logs_bloom_from_logs,
                get_logs_from_receipt_json,
            },
        },
    },
};

pub fn parse_eth_receipt_json(
    eth_receipt_json: EthReceiptJson
) -> Result<EthReceipt> {
    let logs = get_logs_from_receipt_json(&eth_receipt_json)?;
    Ok(
        EthReceipt {
            status: eth_receipt_json.status,
            logs_bloom: get_logs_bloom_from_logs(&logs)?,
            gas_used: U256::from(eth_receipt_json.gasUsed),
            from: convert_hex_to_address(&eth_receipt_json.from)?,
            block_number: U256::from(eth_receipt_json.blockNumber),
            block_hash: convert_hex_to_h256(&eth_receipt_json.blockHash)?,
            transaction_index: U256::from(eth_receipt_json.transactionIndex),
            cumulative_gas_used: U256::from(eth_receipt_json.cumulativeGasUsed),
            transaction_hash: convert_hex_to_h256(&eth_receipt_json.transactionHash)?,
            to: match eth_receipt_json.to {
                serde_json::Value::Null => H160::zero(),
                _ => convert_hex_to_address(&convert_json_value_to_string(eth_receipt_json.to)?)?,
            },
            contract_address: match eth_receipt_json.contractAddress {
                serde_json::Value::Null => Address::zero(),
                _ => convert_hex_to_address(&convert_json_value_to_string(eth_receipt_json.contractAddress)?)?,
            },
            logs,
        }
    )
}

pub fn parse_eth_receipt_jsons(
    eth_receipts_jsons: Vec<EthReceiptJson>
) -> Result<EthReceipts> {
    trace!("✔ Parsing ETH receipt JSON...");
    eth_receipts_jsons.iter().cloned().map(parse_eth_receipt_json).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_expected_receipt,
        SAMPLE_RECEIPT_INDEX,
        get_sample_eth_block_and_receipts_json,
    };

    #[test]
    fn should_parse_eth_receipt_json() {
        let eth_json = get_sample_eth_block_and_receipts_json().unwrap();
        let receipt_json = eth_json.receipts[SAMPLE_RECEIPT_INDEX].clone();
        match parse_eth_receipt_json(receipt_json) {
            Ok(receipt) => assert_eq!(receipt, get_expected_receipt()),
            _ => panic!("Should have parsed receipt!"),
        }
    }

    #[test]
    fn should_parse_eth_receipt_jsons() {
        let eth_json = get_sample_eth_block_and_receipts_json().unwrap();
        if parse_eth_receipt_jsons(eth_json.receipts).is_err() {
            panic!("Should have generated receipts correctly!")
        }
    }
}
