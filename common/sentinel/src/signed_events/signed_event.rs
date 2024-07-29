use common::{crypto_utils::sha256_hash_bytes, types::Bytes, utils::left_pad_bytes_with_zeroes};
use common_chain_ids::EthChainId;
use common_eth::{EthLog, EthPrivateKey, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use common_network_ids::ProtocolId;
use derive_getters::Getters;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{EventIdError, SignedEventError, SignedEventVersion};
use crate::MerkleProof;

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct SignedEvent {
    version: SignedEventVersion,
    protocol: ProtocolId,
    origin: EthChainId,
    log: EthLog,
    tx_id_hash: EthHash,
    block_id_hash: EthHash,
    // NOTE: String in case format changes, plus can't auto derive ser/de on [u8; 65]
    // It's an option so we can create the struct with no signature then add it later.
    event_payload: Option<String>,
    event_id: Option<String>,
    signature: Option<String>,
    public_key: String,
}

const CHAIN_ID_PADDING: usize = 32;
const EVENT_ADDRESS_PADDING: usize = 32;

impl SignedEvent {
    pub(super) fn new(
        metadata_chain_id: MetadataChainId,
        log: EthLog,
        tx_id_hash: EthHash,
        block_id_hash: EthHash,
        pk: &EthPrivateKey,
        // FIXME reintroduce
        _merkle_proof: MerkleProof,
    ) -> Result<Self, SignedEventError> {
        let public_key = pk.to_public_key().public_key.to_string();
        let mut signed_event = Self {
            version: SignedEventVersion::current(),
            protocol: metadata_chain_id.to_protocol_id().into(),
            origin: metadata_chain_id.to_eth_chain_id()?,
            log,
            tx_id_hash,
            block_id_hash,
            event_payload: None,
            event_id: None,
            signature: None,
            public_key,
        };
        let event_payload = signed_event.get_event_payload()?;
        signed_event.event_payload = Some(hex::encode(event_payload));
        let event_id = signed_event.calculate_event_id()?;
        signed_event.event_id = Some(event_id.to_string());
        let sig = pk.sha256_hash_and_sign_msg_with_normalized_parity(&event_id.0)?;
        signed_event.signature = Some(sig.to_string());
        Ok(signed_event)
    }

    fn get_event_payload(&self) -> Result<Bytes, SignedEventError> {
        let address = left_pad_bytes_with_zeroes(self.log.address.as_bytes(), EVENT_ADDRESS_PADDING);
        let topics = self
            .log
            .topics
            .iter()
            .map(|t| t.as_bytes().to_vec())
            .collect::<Vec<_>>()
            .concat();
        // let event_bytes = eth_abi_decode(&[EthAbiParamType::Tuple(vec![EthAbiParamType::Bytes])], &self.log.data)?;
        // let event_bytes = match &event_bytes[0] {
        //     EthAbiToken::Tuple(value) => match &value[0] {
        //         EthAbiToken::Bytes(bytes) => Ok(bytes.to_vec()),
        //         _ => Err(SignedEventError::LogDataError("bad log data format".to_string())),
        //     },
        //     _ => Err(SignedEventError::LogDataError("bad log data format".to_string())),
        // }?;
        

        Ok([address, sha256_hash_bytes(&topics), self.log.data.to_vec()].concat())
    }

    fn calculate_event_id(&self) -> Result<EventId, EventIdError> {
        let event_payload = self.event_payload.as_ref().ok_or(EventIdError::EncodedEventIsNone)?;
        let event_id = [
            self.version.as_bytes(),
            &[self.protocol.into()],
            &left_pad_bytes_with_zeroes(&self.origin.to_bytes()?, CHAIN_ID_PADDING),
            self.block_id_hash.as_bytes(),
            self.tx_id_hash.as_bytes(),
            &hex::decode(event_payload)?,
        ]
        .concat();

        Ok(EventId(event_id))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventId(pub Bytes);

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethereum_types::Address as EthAddress;

    use super::*;

    #[test]
    fn should_create_a_signed_event_correctly() {
        let metadata_chain_id = MetadataChainId::EthereumMainnet;
        let address = EthAddress::from_str("0x87415715056da7a5eb1a30e53c4f4d20b44db71d").unwrap();
        let topics = vec![
            EthHash::from_str("0x9b706941b48091a1c675b439064f40b9d43c577d9c7134cce93179b9b0bf2a52").unwrap(),
            EthHash::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        ];
        let event_bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000ea0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f6652f1db7a7b48d9a6c515ad759c0464e16559c0000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000016345785d8a0000000000000000000000000000ada2de876567a06ed79b0b29ae6ab2e142129e51000000000000000000000000000000000000000000000000000000000000002a30784144413264653837363536376130366544373962304232396165366142326531343231323945353100000000000000000000000000000000000000000000").unwrap();
        let log = EthLog::new(address, topics, event_bytes);
        let block_id_hash =
            EthHash::from_str("0x658d5ae6a577714c7507e7b5911d26429280d6a0922a2be3f4502d577985527a").unwrap();
        let tx_id_hash =
            EthHash::from_str("0x9b3b567ec90fc3a263f1784f57f942ac52ab4e609c23ba794de944fc1b512d34").unwrap();
        let pk = EthPrivateKey::from_str("dfcc79a57e91c42d7eea05f82a08bd1b7e77f30236bb7c56fe98d3366a1929c4").unwrap();
        let merkle_proof = MerkleProof::new(vec![vec![]]);
        let result = SignedEvent::new(metadata_chain_id, log, tx_id_hash, block_id_hash, &pk, merkle_proof).unwrap();

        let expected_signature = "d1820d529a376ed15395faa94a0d8d535620ebc63755a9e57c9cc6d25ac503fa6a14ffa06bc9257ddd3b335da620333fddec0ddf17b151154c3eaa6fe880ff481c".to_string();

        assert_eq!(result.signature.unwrap(), expected_signature);
    }
}
