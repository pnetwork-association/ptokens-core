use std::fmt;

use common::{crypto_utils::keccak_hash_bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use common_network_ids::NetworkId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethabi::Token as EthAbiToken;
use ethereum_types::{H256 as EthHash, U256};
use rs_merkle::{Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use super::{type_aliases::Hash, Actor, Actors, ActorsError};
use crate::{DbKey, DbUtilsT, SentinelDbKeys, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};

type Byte = u8;

#[serde_with::serde_as]
#[derive(Debug, Clone, Eq, Default, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct ActorInclusionProof {
    epoch: U256,
    tx_hash: EthHash,
    #[serde_as(as = "Vec<serde_with::hex::Hex>")]
    proof: Vec<Vec<u8>>,
    network_id: NetworkId,
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
        Ok(Self::new(
            U256::default(),
            EthHash::default(),
            proof,
            NetworkId::default(),
        ))
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

    pub fn update_proof_in_db<D: DatabaseInterface>(&self, db_utils: &SentinelDbUtils<D>) -> Result<(), SentinelError> {
        debug!("maybe updating sentinel inclusion proof in db");
        let existing = Self::get(db_utils);
        let this_epoch = self.epoch();
        let existing_epoch = existing.epoch();
        debug!("this proof's epoch: {this_epoch}");
        debug!("    existing epoch: {existing_epoch}");
        if existing_epoch == &U256::zero() || this_epoch > existing_epoch {
            self.update_in_db(db_utils)
        } else {
            warn!("not updating actors inclusion proof because the epoch is not greater than the existing one");
            Ok(())
        }
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
            "epoch": self.epoch().as_u64(),
            "txHash": format!("0x{}", hex::encode(self.tx_hash())),
            "network_id": self.network_id(),
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
        self.as_merkle_tree().root().unwrap_or_default()
    }

    fn get_inclusion_proof_for_idx(&self, idx: usize) -> Result<ActorInclusionProof, ActorsError> {
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
        Ok(ActorInclusionProof::new(
            *self.epoch(),
            *self.tx_hash(),
            proof,
            *self.network_id(),
        ))
    }

    pub fn get_inclusion_proof_for_actor(&self, actor: &Actor) -> Result<ActorInclusionProof, ActorsError> {
        if let Some(idx) = self.actor_idx(actor) {
            self.get_inclusion_proof_for_idx(idx)
        } else {
            Err(ActorsError::CannotCreateProofForActor(*actor))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::test_utils::get_test_database;
    use ethereum_types::Address as EthAddress;
    use serde_json::json;

    use super::*;
    use crate::{
        actors::test_utils::{get_sample_actors, get_sample_actors_propagated_sub_mat_2},
        Actor,
        ActorType,
    };

    fn get_sample_proof() -> ActorInclusionProof {
        get_sample_actors().get_inclusion_proof_for_idx(1).unwrap()
    }

    #[test]
    fn should_get_actors_merkle_root() {
        let actors = get_sample_actors();
        let expected_root = hex::decode("149b03559160c352fa9f9bc309586f5c8d07d54ae29bd91c9706cb450197bd98").unwrap();
        let root = actors.root().to_vec();
        assert_eq!(root, expected_root);
    }

    fn get_expected_proof() -> ActorInclusionProof {
        ActorInclusionProof::new(
            U256::from(26),
            EthHash::from_str("0xf577503260b8f1c6608d3e50c93895833f783509ae059f1bd0e6f0922720fa67").unwrap(),
            vec![
                hex::decode("d2a063cb44962b73a9fb59d4eefa9be1382810cf6bb85c2769875a86c92ea4b5").unwrap(),
                hex::decode("fec594682ae56dd0b4e447418d170ac775de8a0d49b7f0624a2221daaedb1bb1").unwrap(),
                hex::decode("056b10a893fe384684692e4ae89d2adac2f7b0a3104be865f1ea2e6e8d549e51").unwrap(),
            ],
            NetworkId::try_from("polygon").unwrap(),
        )
    }

    #[test]
    fn should_get_actors_inclusion_proof_by_idx() {
        let actors = get_sample_actors();
        let proof = actors.get_inclusion_proof_for_idx(1).unwrap();
        let expected_proof = get_expected_proof();
        assert_eq!(proof, expected_proof);
    }

    #[test]
    fn should_get_actors_inclusion_proof_by_actor() {
        let actors = get_sample_actors();
        let actor = actors.actors()[1];
        let proof = actors.get_inclusion_proof_for_actor(&actor).unwrap();
        let expected_proof = get_expected_proof();
        assert_eq!(proof, expected_proof);
    }

    #[test]
    fn should_error_getting_inclusion_proof_if_idx_too_high() {
        let actors = get_sample_actors();
        let num_actors = actors.len();
        let idx_to_get_proof_of = num_actors + 1;
        match actors.get_inclusion_proof_for_idx(idx_to_get_proof_of) {
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
        let proof = actors.get_inclusion_proof_for_idx(idx_to_get_proof_of).unwrap();
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
        proof.update_proof_in_db(&db_utils).unwrap();
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

    #[test]
    fn should_err_creating_proof_for_actor_not_amongst_actors() {
        let actor = Actor::new(ActorType::Sentinel, EthAddress::random());
        let actors = get_sample_actors();
        assert!(actors.actor_idx(&actor).is_none());
        match actors.get_inclusion_proof_for_actor(&actor) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(ActorsError::CannotCreateProofForActor(a)) => assert_eq!(actor, a),
            Err(e) => panic!("wrong error received {e}"),
        }
    }

    #[test]
    fn should_get_inclusion_proof() {
        let sub_mat = get_sample_actors_propagated_sub_mat_2();
        let governance_address = EthAddress::from_str("0x94199A50E4DFa680e75C79Ee220E10074E189A95").unwrap();
        let network_id = NetworkId::from_str("polygon").unwrap();
        let actors = Actors::from_sub_mat(&sub_mat, governance_address, network_id)
            .unwrap()
            .unwrap();
        let proof = actors.get_inclusion_proof_for_idx(6).unwrap();
        let expected_proof_json: Json = serde_json::from_str(
            r#"{
            "epoch": "0x29",
            "network_id":{"chain_id":137,"disambiguator":0,"protocol_id":"Ethereum","version":"V1"},
            "tx_hash":"52d8331beb1dc65f13373d2dd531c54903c12170e619c18ecaf2e5971372f7f6",
            "proof":[
                "557f5bca1a37f8d1f3e0d0bf4b6b8d0363f8a8f8cf4f0e02b039057abe54730a",
                "b5fc7273138782204e413820c77222d74beef58b8befd7d19aa64014877fe95f"
            ]
        }"#,
        )
        .unwrap();
        let expected_proof = ActorInclusionProof::try_from(expected_proof_json).unwrap();
        assert_eq!(proof, expected_proof);
    }
}
