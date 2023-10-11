use common::crypto_utils::keccak_hash_bytes;
use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use rs_merkle::{Hasher, MerkleTree}; //, algorithms::Sha256};

use super::ActorType;

type Hash = [u8; 32];

#[derive(Clone, Debug, Deref, Constructor)]
pub struct Actors(Vec<Actor>);

#[derive(Clone, Debug, Constructor, Getters)]
pub struct Actor {
    actor_type: ActorType,
    actor_address: EthAddress,
}

impl Actor {
    fn to_leaf(&self) -> Hash {
        keccak_hash_bytes(&[self.actor_address.as_bytes(), self.actor_type.as_bytes()].concat()).into()
    }
}

#[derive(Clone)]
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
    fn to_leaves(&self) -> Vec<Hash> {
        self.iter().map(Actor::to_leaf).collect()
    }

    fn root(&self) -> EthHash {
        EthHash::from_slice(
            &MerkleTree::<Sha256WithOrderingAlgorithm>::from_leaves(&self.to_leaves())
                .root()
                .unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn get_sample_actors() -> Actors {
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

    #[test]
    fn should_get_actors_merkle_root() {
        // NOTE: See here: https://polygonscan.com/tx/0xdeb8d369543e8f79eb1eee9f1b500cceba179eb92e400f254165c2cc4e40f9f1#eventlog
        let actors = get_sample_actors();
        let expected_root =
            EthHash::from_str("1efc2ea8b69f6ef6c9458e6cfea29d6413900925b57fb35deb8b898464811322").unwrap();
        let root = actors.root();
        assert_eq!(root, expected_root);
    }
}
