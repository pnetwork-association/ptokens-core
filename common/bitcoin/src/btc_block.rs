use std::str::FromStr;

use common::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, NoneError, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "ltc"))]
use crate::bitcoin_crate_alias::blockdata::block::BlockHeader as BtcBlockHeader;
#[cfg(feature = "ltc")]
use crate::bitcoin_crate_alias::blockdata::block::Header as BtcBlockHeader;
use crate::{
    bitcoin_crate_alias::{
        blockdata::block::Block as BtcBlock,
        consensus::encode::deserialize as btc_deserialize,
        hash_types::{BlockHash, TxMerkleNode},
        hashes::Hash,
    },
    btc_submission_material::BtcSubmissionMaterialJson,
    deposit_address_info::DepositInfoList,
    BtcState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockAndId {
    pub height: u64,
    pub block: BtcBlock,
    pub id: BlockHash,
    pub deposit_address_list: DepositInfoList,
}

impl BtcBlockAndId {
    pub fn from_json(json: &BtcSubmissionMaterialJson) -> Result<Self> {
        info!("✔ Parsing `BtcBlockAndId` from json...");
        Ok(Self {
            height: json.block.height,
            block: json.to_btc_block()?,
            id: BlockHash::from_str(&json.block.id)?,
            deposit_address_list: DepositInfoList::from_json(&json.deposit_address_list)?,
        })
    }
}

impl FromStr for BtcBlockAndId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_json(&BtcSubmissionMaterialJson::from_str(s)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Deserialize)]
pub struct BtcBlockJson {
    pub bits: u32,
    pub id: String,
    pub nonce: u32,
    pub version: i32,
    pub height: u64,
    pub timestamp: u32,
    pub merkle_root: String,
    pub previousblockhash: String,
}

impl BtcBlockJson {
    #[cfg(not(feature = "ltc"))]
    pub fn to_block_header(&self) -> Result<BtcBlockHeader> {
        info!("✔ Parsing `BtcBlockJson` to `BtcBlockHeader`...");
        Ok(BtcBlockHeader {
            time: self.timestamp,
            bits: self.bits,
            nonce: self.nonce,
            version: self.version,
            merkle_root: TxMerkleNode::from_str(&self.merkle_root)?,
            prev_blockhash: BlockHash::from_str(&self.previousblockhash)?,
        })
    }

    #[cfg(feature = "ltc")]
    pub fn to_block_header(&self) -> Result<BtcBlockHeader> {
        info!("✔ Parsing `BtcBlockJson` to `BtcBlockHeader`...");
        Ok(BtcBlockHeader {
            time: self.timestamp,
            nonce: self.nonce,
            merkle_root: TxMerkleNode::from_str(&self.merkle_root)?,
            prev_blockhash: BlockHash::from_str(&self.previousblockhash)?,
            bits: crate::bitcoin_crate_alias::CompactTarget::from_consensus(self.bits),
            version: crate::bitcoin_crate_alias::block::Version::from_consensus(self.version),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Constructor)]
pub struct BtcBlockInDbFormat {
    pub height: u64,
    pub id: BlockHash,
    pub extra_data: Bytes,
    eos_minting_params: Option<Bytes>,
    pub eth_minting_params: Option<Bytes>,      // FIXME Privatise
    pub btc_on_int_int_tx_infos: Option<Bytes>, // FIXME Privatise
    pub prev_blockhash: BlockHash,
}

impl BtcBlockInDbFormat {
    pub fn get_tx_info_bytes(&self) -> Bytes {
        // NOTE: As of v7.0.0 we're using just the one field to store tx info as bytes.
        // So now the name is meaningless but we can't change it as it would break the
        // serialization for blocks already stored in a db.
        self.btc_on_int_int_tx_infos.clone().unwrap_or_default()
    }

    pub fn get_btc_on_int_int_tx_infos(&self) -> Bytes {
        self.btc_on_int_int_tx_infos.clone().unwrap_or_default()
    }

    pub fn get_eos_minting_param_bytes(&self) -> Result<Option<Bytes>> {
        // TODO rm!
        Ok(self.eos_minting_params.clone())
    }

    pub fn get_btc_on_int_int_tx_infos_bytes(&self) -> Result<Option<Bytes>> {
        if self.btc_on_int_int_tx_infos.is_some() {
            Ok(Some(self.get_btc_on_int_int_tx_infos()))
        } else {
            Ok(None)
        }
    }

    pub fn get_eth_minting_param_bytes(&self) -> Result<Bytes> {
        Ok(self.eth_minting_params.clone().unwrap_or_default())
    }

    pub fn remove_tx_infos(&self) -> Result<Self> {
        let mut mutable_self = self.clone();
        mutable_self.eos_minting_params = None;
        mutable_self.eth_minting_params = None;
        mutable_self.btc_on_int_int_tx_infos = None;
        Ok(mutable_self)
    }

    fn get_prev_block_hash_bytes(&self) -> Bytes {
        self.prev_blockhash.to_vec()
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        let serialized_id = self.id.to_vec();
        Ok(serde_json::to_vec(&SerializedBlockInDbFormat::new(
            serialized_id,
            convert_u64_to_bytes(self.height),
            self.extra_data.clone(),
            self.get_eth_minting_param_bytes()?,
            self.get_eos_minting_param_bytes()?,
            self.get_btc_on_int_int_tx_infos_bytes()?,
            Some(self.get_prev_block_hash_bytes()),
        ))?)
    }

    #[cfg(all(test, not(feature = "ltc")))]
    fn to_bytes_legacy(&self) -> Result<Bytes> {
        let serialized_id = self.id.to_vec();
        Ok(serde_json::to_vec(&SerializedBlockInDbFormatLegacy::new(
            serialized_id,
            convert_u64_to_bytes(self.height),
            self.extra_data.clone(),
            self.get_eth_minting_param_bytes()?,
            self.get_eos_minting_param_bytes()?,
            Some(self.get_prev_block_hash_bytes()),
        ))?)
    }

    pub fn get_db_key(&self) -> Bytes {
        self.id.to_vec()
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        SerializedBlockInDbFormat::from_bytes(bytes).and_then(|serialized_block_in_db_format| {
            Ok(Self::new(
                convert_bytes_to_u64(&serialized_block_in_db_format.height)?,
                BlockHash::from_slice(&serialized_block_in_db_format.id)?,
                serialized_block_in_db_format.extra_data.clone(),
                serialized_block_in_db_format.eos_minting_params.clone(),
                if serialized_block_in_db_format.eth_minting_params.is_empty() {
                    None
                } else {
                    Some(serialized_block_in_db_format.eth_minting_params.clone())
                },
                serialized_block_in_db_format.get_btc_on_int_int_tx_infos()?,
                serialized_block_in_db_format.get_prev_blockhash()?,
            ))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedBlockInDbFormat {
    pub id: Bytes,
    pub height: Bytes,
    pub extra_data: Bytes,
    pub block: Option<Bytes>,
    pub eth_minting_params: Bytes,
    pub eos_minting_params: Option<Bytes>,
    pub btc_on_int_int_tx_infos: Option<Bytes>,
    pub prev_blockhash: Option<Bytes>,
}

impl SerializedBlockInDbFormat {
    pub fn new(
        id: Bytes,
        height: Bytes,
        extra_data: Bytes,
        eth_minting_params: Bytes,
        eos_minting_params: Option<Bytes>,
        btc_on_int_int_tx_infos: Option<Bytes>,
        prev_blockhash: Option<Bytes>,
    ) -> Self {
        Self {
            id,
            height,
            extra_data,
            eth_minting_params,
            eos_minting_params,
            btc_on_int_int_tx_infos,
            block: None,
            prev_blockhash,
        }
    }

    pub fn from_legacy(legacy_struct: &SerializedBlockInDbFormatLegacy) -> Self {
        // NOTE Pre v2.6.0 blocks stored in the DB had the key `minting_params` instead of `eth_minting_params`.
        Self {
            id: legacy_struct.id.clone(),
            block: legacy_struct.block.clone(),
            height: legacy_struct.height.clone(),
            extra_data: legacy_struct.extra_data.clone(),
            prev_blockhash: legacy_struct.prev_blockhash.clone(),
            eth_minting_params: legacy_struct.eth_minting_params.clone(),
            eos_minting_params: legacy_struct.eos_minting_params.clone(),
            btc_on_int_int_tx_infos: None,
        }
    }

    pub fn get_btc_on_int_int_tx_infos(&self) -> Result<Option<Bytes>> {
        let bytes = self.btc_on_int_int_tx_infos.clone().unwrap_or_default();
        if bytes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(bytes))
        }
    }

    pub fn get_prev_blockhash(&self) -> Result<BlockHash> {
        if self.prev_blockhash.is_some() {
            self.prev_blockhash
                .clone()
                .ok_or(NoneError("No `prev_blockhash` in `SerializedBlockInDbFormat`!"))
                .and_then(|bytes| Ok(BlockHash::from_slice(&bytes)?))
        } else {
            // NOTE: Blocks saved into the DB pre core v2.0.0 contain the block itself.
            self.get_block().map(|block| block.header.prev_blockhash)
        }
    }

    fn get_block(&self) -> Result<BtcBlock> {
        self.block
            .clone()
            .ok_or(NoneError("No BTC block in serialized struct!"))
            .and_then(|bytes| Ok(btc_deserialize(&bytes)?))
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        match serde_json::from_slice(bytes) {
            Ok(serialized_block) => Ok(serialized_block),
            Err(_) => Ok(Self::from_legacy(&SerializedBlockInDbFormatLegacy::from_bytes(bytes)?)),
        }
    }
}

pub fn parse_btc_block_and_id_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    BtcBlockAndId::from_json(state.get_btc_submission_json()?).and_then(|block| state.add_btc_block_and_id(block))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedBlockInDbFormatLegacy {
    pub id: Bytes,
    pub height: Bytes,
    pub extra_data: Bytes,
    pub block: Option<Bytes>,
    pub eth_minting_params: Bytes,
    pub eos_minting_params: Option<Bytes>,
    pub prev_blockhash: Option<Bytes>,
}

impl SerializedBlockInDbFormatLegacy {
    #[cfg(all(test, not(feature = "ltc")))]
    pub fn new(
        id: Bytes,
        height: Bytes,
        extra_data: Bytes,
        eth_minting_params: Bytes,
        eos_minting_params: Option<Bytes>,
        prev_blockhash: Option<Bytes>,
    ) -> Self {
        Self {
            id,
            height,
            extra_data,
            eth_minting_params,
            eos_minting_params,
            block: None,
            prev_blockhash,
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice::<Self>(bytes)?)
    }
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use super::*;
    use crate::{
        bitcoin_crate_alias::{
            blockdata::transaction::Transaction as BtcTransaction,
            consensus::encode::deserialize as btc_deserialize,
        },
        test_utils::{get_sample_btc_block_in_db_format, get_sample_btc_submission_material_json},
    };

    #[test]
    fn should_parse_block_and_tx_json_to_struct() {
        let json = get_sample_btc_submission_material_json().unwrap();
        let result = BtcBlockAndId::from_json(&json);
        assert!(result.is_ok());
    }

    #[test]
    fn should_deserialize_tx() {
        let tx_bytes = hex::decode("0200000000010117c33a062c8d0c2ce104c9988599f6ba382ff9f786ad48519425e39af23da9880000000000feffffff022c920b00000000001976a914be8a09363cd4719b1c05b2703797ca890b718b5088acf980d30d000000001600147448bbdfe47ec14f27c68393e766567ac7c9c77102473044022073fc2b43d5c5f56d7bc92b47a28db989e04988411721db96fb0eea6689fb83ab022034b7ce2729e867962891fec894210d0faf538b971d3ae9059ebb34358209ec9e012102a51b8eb0eb8ef6b2a421fb1aae3d7308e6cdae165b90f78074c2493af98e3612c43b0900").unwrap();
        let result = btc_deserialize::<BtcTransaction>(&tx_bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn should_serde_btc_block_in_db_format() {
        let block = get_sample_btc_block_in_db_format().unwrap();
        let serialized_block = block.to_bytes().unwrap();
        let result = BtcBlockInDbFormat::from_bytes(&serialized_block).unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn should_get_block_from_legacy_serialized_format_correctly() {
        let block = get_sample_btc_block_in_db_format().unwrap();
        let serialized_block_legacy = block.to_bytes_legacy().unwrap();
        let result = BtcBlockInDbFormat::from_bytes(&serialized_block_legacy).unwrap();
        assert_eq!(result, block);
    }
}
