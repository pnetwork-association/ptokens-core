use std::fmt;

use common::{crypto_utils::keccak_hash_bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethabi::Token as EthAbiToken;
use ethereum_types::H256 as EthHash;
use rs_merkle::{Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use super::{type_aliases::Hash, Actors, ActorsError};
use crate::{db_utils::SentinelDbKeys, DbKey, DbUtilsT, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};

type Byte = u8;

#[derive(Debug, Clone, Eq, Default, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct ActorInclusionProof {
    tx_hash: EthHash,
    proof: Vec<Vec<u8>>,
    mcid: MetadataChainId,
}

impl TryFrom<Json> for ActorInclusionProof {
    type Error = ActorsError;

    fn try_from(j: Json) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(j)?)
    }
}

impl From<&ActorInclusionProof> for EthAbiToken {
    fn from(p: &ActorInclusionProof) -> Self {
        EthAbiToken::Array(
            p.proof()
                .iter()
                .map(|v| EthAbiToken::FixedBytes(v.clone()))
                .collect::<Vec<EthAbiToken>>(),
        )
    }
}

impl TryFrom<WebSocketMessagesEncodable> for ActorInclusionProof {
    type Error = ActorsError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        match m {
            WebSocketMessagesEncodable::Success(j) => Self::try_from(j),
            other => Err(Self::Error::CannotCreateProofFrom(other.to_string())),
        }
    }
}

#[cfg(test)]
impl TryFrom<Vec<&str>> for ActorInclusionProof {
    type Error = ActorsError;

    fn try_from(v: Vec<&str>) -> Result<Self, Self::Error> {
        let hash_size = ActorInclusionProof::hash_size();
        let proof = v
            .iter()
            .map(|s| {
                let bs = hex::decode(common::strip_hex_prefix(s))?;
                if bs.len() != hash_size {
                    Err(Self::Error::InvalidHashSizeInProof {
                        got: bs.len(),
                        expected: hash_size,
                        element: s.to_string(),
                    })
                } else {
                    Ok(bs)
                }
            })
            .collect::<Result<Vec<Vec<u8>>, Self::Error>>()?;
        Ok(Self::new(EthHash::default(), proof, MetadataChainId::default()))
    }
}

impl DbUtilsT for ActorInclusionProof {
    fn key(&self) -> Result<DbKey, SentinelError> {
        Ok(SentinelDbKeys::get_actor_inclusion_proof_db_key())
    }

    fn sensitivity() -> Option<Byte> {
        MIN_DATA_SENSITIVITY_LEVEL
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl ActorInclusionProof {
    pub fn get<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>) -> Self {
        if let Ok(p) = Self::get_from_db(db_utils, &SentinelDbKeys::get_actor_inclusion_proof_db_key()) {
            p
        } else {
            Self::empty()
        }
    }

    pub fn put<D: DatabaseInterface>(&self, db_utils: &SentinelDbUtils<D>) -> Result<(), SentinelError> {
        self.update_in_db(db_utils)
    }

    #[cfg(test)]
    fn hash_size() -> usize {
        Sha256WithOrderingAlgorithm::hash_size()
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

impl fmt::Display for ActorInclusionProof {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let proof = self
            .proof()
            .iter()
            .map(|h| format!("0x{}", hex::encode(h)))
            .collect::<Vec<String>>();
        let j = json!({
            "proof": proof,
            "txHash": format!("0x{}", hex::encode(self.tx_hash())),
            "chain": self.mcid(),
        });
        write!(f, "{j}")
    }
}

#[derive(Clone, Debug)]
pub struct Sha256WithOrderingAlgorithm {}

impl Hasher for Sha256WithOrderingAlgorithm {
    type Hash = Hash;

    fn hash(data: &[u8]) -> Hash {
        keccak_hash_bytes(data).into()
    }

    fn concat_and_hash(left_hash: &Self::Hash, maybe_right_hash: Option<&Self::Hash>) -> Self::Hash {
        // NOTE: The JS implementation used by other actors has this flag set to true:
        // https://github.com/merkletreejs/merkletreejs/blob/1f3ab59dcfe74488646de1f237a5a2a860aaa579/src/MerkleTree.ts#L128
        // Hence why we're implementing a custom concatter here.
        // The sorting makes for more efficient (gas-wise) merkle proof validation in solidity.
        match maybe_right_hash {
            None => *left_hash, // NOTE: If no sibling, we propagate this hash up the tree
            Some(right_hash) => {
                let concatted = if *left_hash < *right_hash {
                    [*left_hash, *right_hash].concat()
                } else {
                    [*right_hash, *left_hash].concat()
                };
                Self::hash(&concatted)
            },
        }
    }
}

impl Actors {
    fn as_merkle_tree(&self) -> MerkleTree<Sha256WithOrderingAlgorithm> {
        MerkleTree::<Sha256WithOrderingAlgorithm>::from_leaves(&self.to_leaves())
    }

    #[cfg(test)]
    fn root(&self) -> Hash {
        self.as_merkle_tree().root().unwrap_or_default().into()
    }

    pub fn inclusion_proof(&self, idx: usize) -> Result<ActorInclusionProof, ActorsError> {
        // todo!("find idx from actors event"); FIXME FIXME FIXME

        let num_leaves = self.to_leaves().len();

        if idx > num_leaves {
            return Err(ActorsError::CannotCreateInclusionProof {
                idx,
                num_leaves: self.to_leaves().len(),
            });
        };

        let proof_bytes = self.as_merkle_tree().proof(&[idx]).to_bytes();
        let num_bytes = proof_bytes.len();
        let hash_size = Sha256WithOrderingAlgorithm::hash_size();
        let num_hashes = num_bytes / hash_size;
        let mut proof = vec![];
        for i in 0..num_hashes {
            proof.push(proof_bytes[i * hash_size..(i + 1) * hash_size].to_vec())
        }
        Ok(ActorInclusionProof::new(*self.tx_hash(), proof, *self.mcid()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::test_utils::get_test_database;
    use ethereum_types::Address as EthAddress;
    use serde_json::json;

    use super::*;
    use crate::{actors::test_utils::get_sample_actors, Actor, ActorType};

    fn get_sample_proof() -> ActorInclusionProof {
        get_sample_actors().inclusion_proof(1).unwrap()
    }

    #[test]
    fn should_get_actors_merkle_root() {
        let actors = get_sample_actors();
        let expected_root = hex::decode("149b03559160c352fa9f9bc309586f5c8d07d54ae29bd91c9706cb450197bd98").unwrap();
        let root = actors.root().to_vec();
        assert_eq!(root, expected_root);
    }

    #[test]
    fn should_get_actors_inclusion_proof() {
        let actors = get_sample_actors();
        let proof = actors.inclusion_proof(1).unwrap();
        let mcid = MetadataChainId::PolygonMainnet;
        let tx_hash = EthHash::from_str("0xf577503260b8f1c6608d3e50c93895833f783509ae059f1bd0e6f0922720fa67").unwrap();
        let expected_proof = ActorInclusionProof::new(
            tx_hash,
            vec![
                hex::decode("d2a063cb44962b73a9fb59d4eefa9be1382810cf6bb85c2769875a86c92ea4b5").unwrap(),
                hex::decode("fec594682ae56dd0b4e447418d170ac775de8a0d49b7f0624a2221daaedb1bb1").unwrap(),
                hex::decode("056b10a893fe384684692e4ae89d2adac2f7b0a3104be865f1ea2e6e8d549e51").unwrap(),
            ],
            mcid,
        );
        assert_eq!(proof, expected_proof);
    }

    #[test]
    fn should_error_getting_inclusion_proof_if_idx_too_high() {
        let actors = get_sample_actors();
        let num_actors = actors.len();
        let idx_to_get_proof_of = num_actors + 1;
        match actors.inclusion_proof(idx_to_get_proof_of) {
            Ok(proof) => panic!("should not have succeeded to getting proof: {proof}"),
            Err(ActorsError::CannotCreateInclusionProof { idx, num_leaves }) => {
                assert_eq!(idx, idx_to_get_proof_of);
                assert_eq!(num_leaves, num_actors);
            },
            Err(e) => panic!("wrong error receieved: {e}"),
        }
    }

    #[test]
    fn should_get_proof_for_single_tree_leaf() {
        let actor = Actor::new(
            ActorType::from_str("guardian").unwrap(),
            EthAddress::from_str("0x0ef13b2668dbe1b3edfe9ffb7cbc398363b50f79").unwrap(),
        );
        let actors = Actors::default_with_actors(vec![actor]);
        let num_actors = actors.len();
        assert_eq!(num_actors, 1);
        let idx_to_get_proof_of = 0;
        let proof = actors.inclusion_proof(idx_to_get_proof_of).unwrap();
        let expected_proof = ActorInclusionProof::empty();
        assert_eq!(proof, expected_proof);
    }

    #[test]
    fn should_get_inclusion_proof_from_web_socket_message() {
        let proof = get_sample_proof();
        let m = WebSocketMessagesEncodable::Success(json!(proof));
        let r = ActorInclusionProof::try_from(m).unwrap();
        assert_eq!(r, proof);
    }

    #[test]
    fn should_put_and_get_proof_in_db() {
        let proof = get_sample_proof();
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        proof.put(&db_utils).unwrap();
        let result = ActorInclusionProof::get(&db_utils);
        assert_eq!(proof, result);
    }

    #[test]
    fn should_get_empty_proof_from_db_if_none_extant() {
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        let result = ActorInclusionProof::get(&db_utils);
        assert_eq!(result, ActorInclusionProof::empty());
    }
}
