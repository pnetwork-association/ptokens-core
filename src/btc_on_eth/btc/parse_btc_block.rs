use std::str::FromStr;
use bitcoin_hashes::sha256d;
use bitcoin::{
    consensus::encode::deserialize,
    blockdata::{
        block::Block as BtcBlock,
        block::BlockHeader as BtcBlockHeader,
        transaction::Transaction as BtcTransaction,
    },
};
use crate::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    chains::btc::deposit_address_info::DepositAddressInfo,
    btc_on_eth::btc::{
        btc_state::BtcState,
        btc_types::{
            BtcBlockJson,
            BtcBlockAndId,
            DepositInfoList,
            BtcBlockAndTxsJson,
            DepositAddressJsonList,
        },
    },
};

fn parse_btc_block_json_to_block_header(
    btc_block_json: BtcBlockJson
) -> Result<BtcBlockHeader> {
    trace!("✔ Parsing `BtcBlockJson` to `BtcBlockHeader`...");
    Ok(
        BtcBlockHeader::new(
            btc_block_json.timestamp,
            btc_block_json.bits,
            btc_block_json.nonce,
            btc_block_json.version,
            sha256d::Hash::from_str(&btc_block_json.merkle_root)?,
            sha256d::Hash::from_str(&btc_block_json.previousblockhash)?,
        )
    )
}

pub fn parse_btc_block_json_to_btc_block(
    btc_block_json: BtcBlockAndTxsJson
) -> Result<BtcBlock> {
    trace!("✔ Parsing `BtcBlockAndTxsJson` to `BtcBlock`...");
    Ok(
        BtcBlock::new(
            parse_btc_block_json_to_block_header(
                btc_block_json.block.clone()
            )?,
            convert_hex_txs_to_btc_transactions(
                btc_block_json.transactions
            )?
        )
    )
}

pub fn parse_btc_block_string_to_json(
    btc_block_json_string: &str
) -> Result<BtcBlockAndTxsJson> {
    trace!("✔ Parsing JSON string to `BtcBlockAndTxsJson`...");
    match serde_json::from_str(btc_block_json_string) {
        Ok(json) => Ok(json),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

fn convert_hex_tx_to_btc_transaction(hex: String) -> Result<BtcTransaction> {
    Ok(deserialize::<BtcTransaction>(&hex::decode(hex)?)?)
}

fn convert_hex_txs_to_btc_transactions(
    hex_txs: Vec<String>
) -> Result<Vec<BtcTransaction>> {
    hex_txs
        .into_iter()
        .map(convert_hex_tx_to_btc_transaction)
        .collect::<Result<Vec<BtcTransaction>>>()
}

fn parse_deposit_info_jsons_to_deposit_info_list(
    deposit_address_json_list: &DepositAddressJsonList
) -> Result<DepositInfoList> {
    deposit_address_json_list
        .iter()
        .map(DepositAddressInfo::from_json)
        .collect::<Result<DepositInfoList>>()
}

pub fn parse_btc_block_and_tx_json_to_struct(
    btc_block_json: BtcBlockAndTxsJson
) -> Result<BtcBlockAndId> {
    trace!("✔ Parsing `BtcBlockSAndtxsJson` to `BtcBlockAndId`...");
    Ok(
        BtcBlockAndId {
            height: btc_block_json.block.height,
            id: sha256d::Hash::from_str(&btc_block_json.block.id)?,
            deposit_address_list: parse_deposit_info_jsons_to_deposit_info_list(
                &btc_block_json.deposit_address_list,
            )?,
            block: parse_btc_block_json_to_btc_block(
                btc_block_json
            )?,
        }
    )
}

pub fn parse_btc_block_and_id_and_put_in_state<D>(
    block_json: String,
    mut state: BtcState<D>,
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing BTC block...");
    parse_btc_block_string_to_json(&block_json)
        .and_then(|btc_block_json| {
            #[cfg(feature = "any-sender")]
            return state.set_any_sender_from_btc_block_and_tx_json(btc_block_json);

            #[cfg(not(feature = "any-sender"))]
            Ok(btc_block_json)
        })
        .and_then(parse_btc_block_and_tx_json_to_struct)
        .and_then(|result| state.add_btc_block_and_id(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::btc::btc_test_utils::{
        get_sample_btc_block_json,
        get_sample_btc_block_json_string,
    };

    #[test]
    fn should_parse_btc_block_json() {
        let string = get_sample_btc_block_json_string();
        if let Err(e) = parse_btc_block_string_to_json(&string) {
            panic!("Error getting json from btc block and txs sample: {}", e);
        }
    }

    #[test]
    fn should_parse_block_and_tx_json_to_struct() {
        let json = get_sample_btc_block_json()
            .unwrap();
        if let Err(e) = parse_btc_block_and_tx_json_to_struct(json) {
            panic!("Error getting json from btc block and txs sample: {}", e);
        }
    }

    #[test]
    fn should_not_panic_deserializing_tx() {
        let tx_bytes = hex::decode("0200000000010117c33a062c8d0c2ce104c9988599f6ba382ff9f786ad48519425e39af23da9880000000000feffffff022c920b00000000001976a914be8a09363cd4719b1c05b2703797ca890b718b5088acf980d30d000000001600147448bbdfe47ec14f27c68393e766567ac7c9c77102473044022073fc2b43d5c5f56d7bc92b47a28db989e04988411721db96fb0eea6689fb83ab022034b7ce2729e867962891fec894210d0faf538b971d3ae9059ebb34358209ec9e012102a51b8eb0eb8ef6b2a421fb1aae3d7308e6cdae165b90f78074c2493af98e3612c43b0900")
            .unwrap();
        if let Err(e) = deserialize::<BtcTransaction>(&tx_bytes) {
            panic!("Error deserializing tx: {}", e);
        }
    }
}
