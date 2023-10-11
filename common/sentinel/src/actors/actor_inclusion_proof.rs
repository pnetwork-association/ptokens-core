use std::fmt;

use common::{crypto_utils::keccak_hash_bytes, strip_hex_prefix};
use derive_more::{Constructor, Deref};
use ethabi::Token as EthAbiToken;
use rs_merkle::{Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::{type_aliases::Hash, Actors, ActorsError};
use crate::WebSocketMessagesEncodable;

#[derive(Debug, Clone, Eq, Default, PartialEq, Deref, Constructor, Serialize, Deserialize)]
pub struct ActorInclusionProof(Vec<Vec<u8>>);

impl TryFrom<Json> for ActorInclusionProof {
    type Error = ActorsError;

    fn try_from(j: Json) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(j)?)
    }
}

impl From<&ActorInclusionProof> for EthAbiToken {
    fn from(p: &ActorInclusionProof) -> Self {
        EthAbiToken::Array(
            p.iter()
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

impl TryFrom<Vec<&str>> for ActorInclusionProof {
    type Error = ActorsError;

    fn try_from(v: Vec<&str>) -> Result<Self, Self::Error> {
        let hash_size = ActorInclusionProof::hash_size();
        Ok(Self::new(
            v.iter()
                .map(|s| {
                    let bs = hex::decode(strip_hex_prefix(s))?;
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
                .collect::<Result<Vec<Vec<u8>>, Self::Error>>()?,
        ))
    }
}

impl ActorInclusionProof {
    fn hash_size() -> usize {
        Sha256WithOrderingAlgorithm::hash_size()
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self::default()
    }
}

impl fmt::Display for ActorInclusionProof {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ss = self
            .0
            .iter()
            .map(|h| format!("0x{}", hex::encode(h)))
            .collect::<Vec<String>>();
        write!(f, "{ss:?}")
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
        let mut r = vec![];
        for i in 0..num_hashes {
            r.push(proof_bytes[i * hash_size..(i + 1) * hash_size].to_vec())
        }
        Ok(ActorInclusionProof::new(r))
    }
}

#[cfg(test)]
mod tests {
    use ethereum_types::Address as EthAddress;
    use std::str::FromStr;

    use serde_json::json;

    use super::*;
    use crate::{Actor, ActorType};

    fn get_sample_actors() -> Actors {
        // NOTE: See here: https://polygonscan.com/tx/0xdeb8d369543e8f79eb1eee9f1b500cceba179eb92e400f254165c2cc4e40f9f1#eventlog
        Actors::new(vec![
            Actor::new(
                ActorType::from_str("guardian").unwrap(),
                EthAddress::from_str("0x0ef13b2668dbe1b3edfe9ffb7cbc398363b50f79").unwrap(),
            ),
            Actor::new(
                ActorType::from_str("guardian").unwrap(),
                EthAddress::from_str("0xdb30d31ce9a22f36a44993b1079ad2d201e11788").unwrap(),
            ),
            Actor::new(
                ActorType::from_str("sentinel").unwrap(),
                EthAddress::from_str("0xe06c8959f4c10fcaa9a7ff0d4c4acdda2610da22").unwrap(),
            ),
        ])
    }

    fn get_sample_proof() -> ActorInclusionProof {
        get_sample_actors().inclusion_proof(1).unwrap()
    }

    #[test]
    fn should_get_actors_merkle_root() {
        let actors = get_sample_actors();
        let expected_root = hex::decode("1efc2ea8b69f6ef6c9458e6cfea29d6413900925b57fb35deb8b898464811322").unwrap();
        let root = actors.root().to_vec();
        assert_eq!(root, expected_root);
    }

    #[test]
    fn should_get_actors_inclusion_proof() {
        let actors = get_sample_actors();
        let proof = actors.inclusion_proof(1).unwrap();
        let expected_proof = ActorInclusionProof::try_from(vec![
            "0xd2a063cb44962b73a9fb59d4eefa9be1382810cf6bb85c2769875a86c92ea4b5",
            "0x42a6a3a18f1c558fec27b5ea2b184f0c836be9b14a6b75144e70382ee01d6428",
        ])
        .unwrap();
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
        let actors = Actors::new(vec![actor]);
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
}
