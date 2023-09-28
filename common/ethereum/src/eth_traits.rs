use common::{
    crypto_utils::keccak_hash_bytes,
    types::{Byte, Bytes, Result},
};
use ethereum_types::H256 as EthHash;

use crate::{EthSignature, RelayTransaction};

pub trait EthTxInfoCompatible {
    fn is_any_sender(&self) -> bool;
    fn any_sender_tx(&self) -> Option<RelayTransaction>;
    fn eth_tx_hex(&self) -> Option<String>;
    fn serialize_bytes(&self) -> Bytes;
    fn get_tx_hash(&self) -> String {
        hex::encode(keccak_hash_bytes(&self.serialize_bytes()))
    }
}

pub trait EthSigningCapabilities {
    fn sign_hash(&self, hash: EthHash) -> Result<EthSignature>;
    fn hash_and_sign_msg(&self, message: &[Byte]) -> Result<EthSignature>;
    fn hash_and_sign_msg_with_eth_prefix(&self, message: &[Byte]) -> Result<EthSignature>;
    fn sign_hash_and_set_eth_recovery_param(&self, hash: EthHash) -> Result<EthSignature>;
}
