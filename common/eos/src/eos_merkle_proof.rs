use common::AppError;
use derive_more::{Constructor, Deref};
use eos_chain::Checksum256;

use crate::{eos_incremerkle::Incremerkle, eos_utils::convert_hex_to_checksum256};

#[derive(Debug, Clone, Default, Eq, PartialEq, Constructor, Deref)]
pub(crate) struct MerkleProof(Vec<String>);

impl From<Vec<String>> for MerkleProof {
    fn from(v: Vec<String>) -> Self {
        Self::new(v)
    }
}

impl MerkleProof {
    pub(crate) fn hash_canonical_pair(pair: (Checksum256, Checksum256)) -> Result<Checksum256, AppError> {
        Ok(Checksum256::hash(pair)?)
    }

    fn make_and_hash_canonical_pair(l: &Checksum256, r: &Checksum256) -> Result<Checksum256, AppError> {
        Self::hash_canonical_pair(Incremerkle::make_canonical_pair(l, r))
    }

    pub(crate) fn verify(&self) -> Result<bool, AppError> {
        let mut node = convert_hex_to_checksum256(&self[0])?;
        let leaves = self[..self.len() - 1]
            .iter()
            .map(convert_hex_to_checksum256)
            .collect::<Result<Vec<Checksum256>, AppError>>()?;
        for leaf in leaves.iter().skip(1) {
            if Incremerkle::is_canonical_right(leaf) {
                node = Self::make_and_hash_canonical_pair(&node, leaf)?;
            } else {
                node = Self::make_and_hash_canonical_pair(leaf, &node)?;
            }
        }
        let last_str = match self.last() {
            Some(s) => s.to_string(),
            _ => "".to_string(),
        };
        let last = convert_hex_to_checksum256(last_str)?;
        Ok(node == last)
    }
}

#[cfg(test)]
mod tests {
    use common::types::Byte;

    use super::*;
    use crate::eos_test_utils::get_sample_eos_submission_material_n;

    fn get_expected_first_byte_1() -> Byte {
        0b0001_1011
    }

    fn get_expected_first_byte_2() -> Byte {
        0b1001_0010
    }

    fn get_expected_digest_hex_1() -> &'static str {
        "9b9babebfbdff48ce4002b5f3c7f999c0ee74707b6d121c47ef5db68c6be7262"
    }

    fn get_expected_digest_hex_2() -> &'static str {
        "122cd09d66ca7df007a35bd9c9be5484833f1a69ad0c8527c3e2a56b6955e761"
    }

    fn get_expected_digest_1() -> Checksum256 {
        convert_hex_to_checksum256(get_expected_digest_hex_1()).unwrap()
    }

    fn get_expected_digest_2() -> Checksum256 {
        convert_hex_to_checksum256(get_expected_digest_hex_2()).unwrap()
    }

    fn get_sample_canonical_pair() -> (Checksum256, Checksum256) {
        Incremerkle::make_canonical_pair(&get_expected_digest_1(), &get_expected_digest_2())
    }

    #[test]
    fn should_make_canonical_pair() {
        let digest_1 = get_expected_digest_1();
        let digest_2 = get_expected_digest_2();
        let (left, right) = Incremerkle::make_canonical_pair(&digest_1, &digest_2);

        for i in 0..left.0.len() {
            if i == 0 {
                assert_eq!(left.0[i], get_expected_first_byte_1());
            } else {
                assert_eq!(left.0[i], digest_1.0[i]);
            }
        }
        for i in 0..right.0.len() {
            if i == 0 {
                assert_eq!(right.0[i], get_expected_first_byte_2());
            } else {
                assert_eq!(right.0[i], digest_2.0[i]);
            }
        }
    }

    #[test]
    fn should_hash_canonical_pair() {
        let expected_result = "a26284468e89fe4a5cce763ca3b3d3d37d5fcb35f289c63f0558487ec57ace28";
        let canonical_pair = get_sample_canonical_pair();
        let result = MerkleProof::hash_canonical_pair(canonical_pair).unwrap();
        assert_eq!(result.to_string(), expected_result);
    }

    #[test]
    fn should_verify_merkle_proofs() {
        let num_proofs = 4;
        [0, num_proofs - 1]
            .iter()
            .map(|i| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material| submission_material.action_proofs[0].action_proof.clone().into())
            .for_each(|proof: MerkleProof| assert!(proof.verify().unwrap()));
    }
}
