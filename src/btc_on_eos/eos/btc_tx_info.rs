use std::str::from_utf8;

use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
        eos::{
            eos_action_proofs::EosActionProof,
            eos_global_sequences::{GlobalSequence, GlobalSequences, ProcessedGlobalSequences},
            eos_state::EosState,
        },
    },
    constants::FEE_BASIS_POINTS_DIVISOR,
    fees::fee_utils::sanity_check_basis_points_value,
    traits::DatabaseInterface,
    types::Result,
    utils::convert_bytes_to_u64,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct BtcOnEosBtcTxInfos(pub Vec<BtcOnEosBtcTxInfo>);

impl BtcOnEosBtcTxInfos {
    pub fn subtract_fees(&self, fee_basis_points: u64) -> Result<Self> {
        let (fees, _) = self.calculate_fees(sanity_check_basis_points_value(fee_basis_points)?);
        info!("`BtcOnEosBtcTxInfos` fees: {:?}", fees);
        Ok(Self::new(
            fees.iter()
                .zip(self.iter())
                .map(|(fee, btc_tx_info)| btc_tx_info.subtract_amount(*fee))
                .collect::<Result<Vec<BtcOnEosBtcTxInfo>>>()?,
        ))
    }

    pub fn calculate_fees(&self, basis_points: u64) -> (Vec<u64>, u64) {
        info!("✔ Calculating fees in `BtcOnEosBtcTxInfos`...");
        let fees = self
            .iter()
            .map(|btc_tx_info| btc_tx_info.calculate_fee(basis_points))
            .collect::<Vec<u64>>();
        let total_fee = fees.iter().sum();
        info!("✔      Fees: {:?}", fees);
        info!("✔ Total fee: {:?}", fees);
        (fees, total_fee)
    }

    pub fn sum(&self) -> u64 {
        self.0.iter().fold(0, |acc, infos| acc + infos.amount)
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        GlobalSequences::new(
            self.0
                .iter()
                .map(|infos| infos.global_sequence)
                .collect::<Vec<GlobalSequence>>(),
        )
    }

    pub fn from_action_proofs(action_proofs: &[EosActionProof]) -> Result<BtcOnEosBtcTxInfos> {
        Ok(BtcOnEosBtcTxInfos::new(
            action_proofs
                .iter()
                .map(BtcOnEosBtcTxInfo::from_action_proof)
                .collect::<Result<Vec<BtcOnEosBtcTxInfo>>>()?,
        ))
    }

    pub fn filter_out_already_processed_txs(
        &self,
        processed_tx_ids: &ProcessedGlobalSequences,
    ) -> Result<BtcOnEosBtcTxInfos> {
        Ok(BtcOnEosBtcTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<BtcOnEosBtcTxInfo>>(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEosBtcTxInfo {
    pub amount: u64,
    pub recipient: String,
    pub from: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
}

impl BtcOnEosBtcTxInfo {
    pub fn subtract_amount(&self, subtrahend: u64) -> Result<Self> {
        info!("✔ Subtracting {} from `BtcOnEosBtcTxInfo`...", subtrahend);
        if subtrahend > self.amount {
            Err(format!("Cannot subtract {} from {}!", subtrahend, self.amount).into())
        } else {
            let new_amount = self.amount - subtrahend;
            info!(
                "Subtracted amount of {} from current BTC tx info amount of {} to get final amount of {}",
                subtrahend, self.amount, new_amount
            );
            Ok(Self {
                from: self.from,
                amount: new_amount,
                recipient: self.recipient.clone(),
                global_sequence: self.global_sequence,
                originating_tx_id: self.originating_tx_id,
            })
        }
    }

    pub fn calculate_fee(&self, basis_points: u64) -> u64 {
        (self.amount * basis_points) / FEE_BASIS_POINTS_DIVISOR
    }

    pub fn get_eos_amount_from_proof(proof: &EosActionProof) -> Result<u64> {
        proof
            .check_proof_action_data_length(15, "Not enough data to parse `BtcOnEosBtcTxInfo` amount from proof!")
            .and_then(|_| convert_bytes_to_u64(&proof.action.data[8..=15]))
    }

    pub fn get_action_sender_from_proof(proof: &EosActionProof) -> Result<EosAccountName> {
        proof
            .check_proof_action_data_length(7, "Not enough data to parse `BtcOnEosBtcTxInfo` sender from proof!")
            .and_then(|_| {
                let result = EosAccountName::new(convert_bytes_to_u64(&proof.action.data[..=7])?);
                debug!("✔ Account name parsed from redeem action: {}", result);
                Ok(result)
            })
    }

    pub fn get_redeem_address_from_proof(proof: &EosActionProof) -> Result<String> {
        proof
            .check_proof_action_data_length(25, "Not enough data to parse `BtcOnEosBtcTxInfo` redeemer from proof!")
            .and_then(|_| Ok(from_utf8(&proof.action.data[25..])?.to_string()))
    }

    pub fn from_action_proof(proof: &EosActionProof) -> Result<Self> {
        info!("✔ Converting action proof to `btc-on-eos` BTC tx info...");
        Ok(Self {
            originating_tx_id: proof.tx_id,
            amount: Self::get_eos_amount_from_proof(proof)?,
            from: Self::get_action_sender_from_proof(proof)?,
            global_sequence: proof.action_receipt.global_sequence,
            recipient: Self::get_redeem_address_from_proof(proof)?,
        })
    }
}

pub fn maybe_parse_btc_tx_infos_and_put_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Parsing BTC tx infos from actions data...");
    BtcOnEosBtcTxInfos::from_action_proofs(&state.action_proofs).and_then(|btc_tx_infos| {
        info!("✔ Parsed {} sets of BTC tx info!", btc_tx_infos.len());
        state.add_btc_on_eos_btc_tx_infos(btc_tx_infos)
    })
}

pub fn filter_out_value_too_low_btc_on_eos_btc_tx_infos(
    btc_tx_infos: &BtcOnEosBtcTxInfos,
) -> Result<BtcOnEosBtcTxInfos> {
    Ok(BtcOnEosBtcTxInfos::new(
        btc_tx_infos
            .iter()
            .map(|btc_tx_info| btc_tx_info.amount)
            .zip(btc_tx_infos.0.iter())
            .filter_map(|(amount, btc_tx_info)| match amount >= MINIMUM_REQUIRED_SATOSHIS {
                true => Some(btc_tx_info),
                false => {
                    info!("✘ Filtering redeem BTC tx info ∵ value too low: {:?}", btc_tx_info);
                    None
                },
            })
            .cloned()
            .collect::<Vec<BtcOnEosBtcTxInfo>>(),
    ))
}

pub fn maybe_filter_value_too_low_btc_tx_infos_in_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out any BTC tx infos below minimum # of Satoshis...");
    filter_out_value_too_low_btc_on_eos_btc_tx_infos(&state.btc_on_eos_btc_tx_infos)
        .and_then(|new_infos| state.replace_btc_on_eos_btc_tx_infos(new_infos))
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out already processed tx IDs...");
    state
        .btc_on_eos_btc_tx_infos
        .filter_out_already_processed_txs(&state.processed_tx_ids)
        .and_then(|filtered| state.add_btc_on_eos_btc_tx_infos(filtered))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        btc_on_eos::test_utils::{get_sample_btc_tx_info, get_sample_btc_tx_infos},
        chains::eos::{eos_test_utils::get_sample_eos_submission_material_n, eos_utils::convert_hex_to_checksum256},
        errors::AppError,
    };

    #[test]
    fn should_get_amount_from_proof() {
        let proof = &get_sample_eos_submission_material_n(1).action_proofs[0].clone();
        let expected_result: u64 = 5111;
        let result = BtcOnEosBtcTxInfo::get_eos_amount_from_proof(proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_sender_from_proof() {
        let proof = &get_sample_eos_submission_material_n(1).action_proofs[0].clone();
        let expected_result = EosAccountName::from_str("provtestable").unwrap();
        let result = BtcOnEosBtcTxInfo::get_action_sender_from_proof(proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_address_from_proof() {
        let proof = &get_sample_eos_submission_material_n(1).action_proofs[0].clone();
        let expected_result = "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM";
        let result = BtcOnEosBtcTxInfo::get_redeem_address_from_proof(proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_btc_tx_infos_from_action_proof_2() {
        let expected_result = BtcOnEosBtcTxInfo {
            global_sequence: 577606126,
            amount: 1,
            recipient: "mr6ioeUxNMoavbr2VjaSbPAovzzgDT7Su9".to_string(),
            from: EosAccountName::from_str("provabletest").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                "34dff748d2bbb9504057d4be24c69b8ac38b2905f7e911dd0e9ed3bf369bae03",
            )
            .unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(2).action_proofs[0].clone();
        let result = BtcOnEosBtcTxInfo::from_action_proof(&action_proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_btc_tx_infos_from_action_proof_3() {
        let expected_result = BtcOnEosBtcTxInfo {
            global_sequence: 583774614,
            amount: 5666,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            from: EosAccountName::from_str("provabletest").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                "51f0dbbaf6989e9b980d0fa18bd70ddfc543851ff65140623d2cababce2ceb8c",
            )
            .unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(3).action_proofs[0].clone();
        let result = BtcOnEosBtcTxInfo::from_action_proof(&action_proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_btc_tx_infos_from_action_proof_4() {
        let expected_result = BtcOnEosBtcTxInfo {
            global_sequence: 579818529,
            amount: 5555,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            from: EosAccountName::from_str("provtestable").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                "8eaafcb796002a12e0f48ebc0f832bacca72a8b370e00967c65619a2c1814a04",
            )
            .unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(4).action_proofs[0].clone();
        let result = BtcOnEosBtcTxInfo::from_action_proof(&action_proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_btc_tx_infos_from_action_proof_5() {
        let expected_result = BtcOnEosBtcTxInfo {
            global_sequence: 579838915,
            amount: 5111,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            from: EosAccountName::from_str("provtestable").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                "aebe7cd1a4687485bc5db87bfb1bdfb44bd1b7f9c080e5cb178a411fd99d2fd5",
            )
            .unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(1).action_proofs[0].clone();
        let result = BtcOnEosBtcTxInfo::from_action_proof(&action_proof).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fee_in_btc_on_eos_redeem_param() {
        let infos = get_sample_btc_tx_info();
        let basis_points = 25;
        let result = infos.calculate_fee(basis_points);
        let expected_result = 12;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fee_in_btc_on_eos_btc_tx_infos() {
        let infos = get_sample_btc_tx_infos();
        let basis_points = 25;
        let (fees, total_fee) = infos.calculate_fees(basis_points);
        let expected_fees = vec![12, 12];
        let expected_total_fee: u64 = expected_fees.iter().sum();
        assert_eq!(total_fee, expected_total_fee);
        assert_eq!(fees, expected_fees);
    }

    #[test]
    fn should_subtract_amount_from_btc_on_eos_btc_tx_infos() {
        let infos = get_sample_btc_tx_info();
        let subtrahend = 1337;
        let result = infos.subtract_amount(subtrahend).unwrap();
        let expected_result = 3774;
        assert_eq!(result.amount, expected_result)
    }

    #[test]
    fn should_subtract_fees_from_btc_on_eos_btc_tx_infos() {
        let infos = get_sample_btc_tx_infos();
        let basis_points = 25;
        let result = infos.subtract_fees(basis_points).unwrap();
        let expected_amount = 5099;
        result.iter().for_each(|info| assert_eq!(info.amount, expected_amount));
    }

    #[test]
    fn should_fail_to_subtact_too_large_an_amount_from_btc_on_eos_btc_tx_info() {
        let info = get_sample_btc_tx_infos()[0].clone();
        let subtrahend = info.amount + 1;
        let expected_err = format!("Cannot subtract {} from {}!", subtrahend, info.amount);
        match info.subtract_amount(subtrahend) {
            Ok(_) => panic!("Should not have suceeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error received!"),
        };
    }
}
