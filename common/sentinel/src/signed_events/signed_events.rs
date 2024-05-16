use common_eth::{EthLog, EthPrivateKey, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::{SignedEvent, SignedEventError};
use crate::{ConfiguredEvent, MerkleProof, MerkleTree, NetworkConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct SignedEvents(Vec<SignedEvent>);

impl SignedEvents {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl From<Vec<SignedEvents>> for SignedEvents {
    fn from(vec_of_signed_events: Vec<SignedEvents>) -> Self {
        let mut r: SignedEvents = SignedEvents::empty();
        for signed_events in vec_of_signed_events.into_iter() {
            for signed_event in signed_events.0.into_iter() {
                r.push(signed_event)
            }
        }
        r
    }
}

impl TryFrom<(&MetadataChainId, &EthPrivateKey, &EthSubmissionMaterial, &NetworkConfig)> for SignedEvents {
    type Error = SignedEventError;

    fn try_from(
        (metadata_chain_id, private_key, eth_submission_material, network_config): (
            &MetadataChainId,
            &EthPrivateKey,
            &EthSubmissionMaterial,
            &NetworkConfig,
        ),
    ) -> Result<Self, Self::Error> {
        let block_hash = eth_submission_material.get_block_hash()?;
        let mut merkle_tree = MerkleTree::try_from(eth_submission_material)?;
        let mut relevant_infos: Vec<(MerkleProof, Vec<EthLog>)> = vec![];

        // NOTE: These are the events that the sentinel is configured to watch out for (via the config file)
        for ConfiguredEvent { address, topic } in network_config.events().iter() {
            for receipt in eth_submission_material.receipts.iter() {
                let mut relevant_logs = vec![];

                for log in receipt.logs.iter() {
                    if log.is_from_address_and_contains_topic(address, topic) {
                        relevant_logs.push(log.clone())
                    };
                }

                if relevant_logs.is_empty() {
                    continue;
                } else {
                    debug!("found {} relevant logs", relevant_logs.len());
                    let receipt_inclusion_proof = MerkleProof::try_from((&mut merkle_tree, &receipt.transaction_hash))?;
                    relevant_infos.push((receipt_inclusion_proof, relevant_logs.clone()));
                    relevant_logs.clear();
                }
            }
        }

        let mut signed_events = vec![];
        for (receipt_inclusion_proof, logs) in relevant_infos.into_iter() {
            for log in logs.into_iter() {
                let signed_event = SignedEvent::new(
                    log,
                    block_hash,
                    receipt_inclusion_proof.clone(),
                    *metadata_chain_id,
                    private_key,
                )?;
                signed_events.push(signed_event);
            }
        }

        Ok(Self::new(signed_events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_signed_events() {
        todo!("write this test");
    }
}
