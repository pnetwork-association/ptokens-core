use common_eth::{EthLog, EthPrivateKey, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{SignedEventError, SignedEventVersion};
use crate::MerkleProof;

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct SignedEvent {
    log: EthLog,
    block_hash: EthHash,
    // NOTE: String in case format changes, plus can't auto derive ser/de on [u8; 65]
    // It's an option so we can create the struct with no signature then add it later.
    signature: Option<String>,
    merkle_proof: MerkleProof,
    version: SignedEventVersion,
    // NOTE: gitmp needs this instead of the v3 NetworkId
    metadata_chain_id: MetadataChainId,
}

impl SignedEvent {
    pub(super) fn new(
        log: EthLog,
        block_hash: EthHash,
        merkle_proof: MerkleProof,
        metadata_chain_id: MetadataChainId,
        pk: &EthPrivateKey,
    ) -> Result<Self, SignedEventError> {
        let mut signed_event = Self {
            log,
            block_hash,
            merkle_proof,
            signature: None,
            metadata_chain_id,
            version: SignedEventVersion::current(),
        };
        let hash = signed_event.encode()?;
        let sig = pk.sha256_hash_and_sign_msg(hash.0.as_slice())?;
        signed_event.signature = Some(sig.to_string());
        Ok(signed_event)
    }

    fn encode(&self) -> Result<EthHash, SignedEventError> {
        todo!("encode via gitmp01's spec");
    }
}
