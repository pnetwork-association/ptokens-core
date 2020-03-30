use eos_primitives::Checksum256;
use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    utils::convert_bytes_to_checksum256,
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
    },
};

fn filter_out_proofs_with_action_digests_not_in_action_receipts(
    action_proofs: &ActionProofs
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .map(|proof| proof.action.to_digest())
            .map(|digest_bytes| convert_bytes_to_checksum256(&digest_bytes))
            .collect::<Result<Vec<Checksum256>>>()?
            .into_iter()
            .zip(action_proofs.iter())
            .filter(|(digest, proof)| digest == &proof.action_receipt.act_digest)
            .map(|(_, proof)| proof)
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_action_proof_receipt_mismatches<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering proofs w/ action digests NOT in action receipts...");
    filter_out_proofs_with_action_digests_not_in_action_receipts(
        &state.action_proofs
    )
        .and_then(|proofs| state.replace_action_proofs(proofs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::{
        eos::eos_test_utils::get_sample_action_proof_n,
    };

    #[test]
    fn should_not_filter_out_proofs_with_action_digests_not_in_action_receipts() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(5),
        ];
        let result = filter_out_proofs_with_action_digests_not_in_action_receipts(&action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_with_action_digests_not_in_action_receipts() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(5),
        ];

        let mut proof_with_invalid_action = get_sample_action_proof_n(3);
        proof_with_invalid_action.action.data[0] = 42;

        let mut dirty_action_proofs = vec![proof_with_invalid_action];
        dirty_action_proofs.extend_from_slice(&action_proofs);
        
        let result = filter_out_proofs_with_action_digests_not_in_action_receipts(&dirty_action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }
}
