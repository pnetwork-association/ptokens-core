use std::str::{
    FromStr,
    from_utf8,
};
use eos_primitives::{
    Symbol as EosSymbol,
    AccountName as EosAccountName,
};
use crate::btc_on_eos::{
    traits::DatabaseInterface,
    utils::convert_bytes_to_u64,
    btc::btc_constants::BTC_NUM_DECIMALS,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_state::EosState,
        eos_types::{
            ActionData,
            ActionsData,
            RedeemParams,
        },
    },
};

fn get_eos_symbol_from_serialized_action(
    serialized_action: &Bytes
) -> Result<EosSymbol> {
    Ok(
        EosSymbol::new(
            convert_bytes_to_u64(&serialized_action[50..58].to_vec())?
        )
    )
}

fn get_eos_amount_from_serialized_action(
    serialized_action: &Bytes
) -> Result<u64> {
    convert_bytes_to_u64(&serialized_action[42..50].to_vec())
}

fn get_eos_action_name_from_serialized_action(
    serialized_action: &Bytes
) -> Result<EosAccountName> {
    Ok(
        EosAccountName::new(
            convert_bytes_to_u64(&serialized_action[8..16].to_vec())?
        )
    )
}

fn get_redeem_action_sender_from_serialize_action(
    serialized_action: &Bytes
) -> Result<EosAccountName> {
    Ok(
        EosAccountName::new(
            convert_bytes_to_u64(&serialized_action[34..42].to_vec())?
        )
    )
}

fn get_redeem_address_from_serialized_action(
    serialized_action: &Bytes,
) -> Result<String> {
    Ok(from_utf8(&serialized_action[59..])?.to_string())
}

impl RedeemParams {
    pub fn from_action_data(
        action_data: &ActionData,
    ) -> Result<Self> {
        Ok(
            RedeemParams {
                amount: get_eos_amount_from_serialized_action(
                    &action_data.action_proof.serialized_action,
                )?,
                from: get_redeem_action_sender_from_serialize_action(
                    &action_data.action_proof.serialized_action,
                )?,
                recipient: get_redeem_address_from_serialized_action(
                    &action_data.action_proof.serialized_action,
                )?,
                originating_tx_id: action_data.action_proof.tx_id.clone(),
            }
        )
    }
}

pub fn parse_redeem_params_from_actions_data(
    actions_data: &ActionsData
) -> Result<Vec<RedeemParams>> {
    actions_data
        .iter()
        .map(|action_data| RedeemParams::from_action_data(action_data))
        .collect()
}

// TODO Filter to ensure the symbol is correct!
// TODO Filter to ensure they're `redeem` actions
pub fn maybe_parse_redeem_params_and_put_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing redeem params from actions data...");
    parse_redeem_params_from_actions_data(&state.actions_data)
        .and_then(|params| {
            debug!("✔ Parsed {} sets of params!", params.len());
            state.add_redeem_params(params)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::{
        utils::convert_hex_to_checksum256,
        eos::eos_test_utils::get_sample_eos_submission_material_n,
    };

    #[test]
    fn should_get_sender_from_serialized_action() {
        let expected_result = EosAccountName::from_str("provtestable")
            .unwrap();
        let serialized_action = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .serialized_action
            .clone();
        let result = get_redeem_action_sender_from_serialize_action(
            &serialized_action
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_symbol_from_serialized_action() {
        let expected_result = EosSymbol::from_str("8,PFFF")
            .unwrap();
        let serialized_action = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .serialized_action
            .clone();
        let result = get_eos_symbol_from_serialized_action(&serialized_action)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_amount_from_serialized_action() {
        let expected_result: u64 = 5111;
        let serialized_action = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .serialized_action
            .clone();
        let result = get_eos_amount_from_serialized_action(&serialized_action)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_action_name_from_serialized_action() {
        let expected_result = EosAccountName::from_str("redeem")
            .unwrap();
        let serialized_action = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .serialized_action
            .clone();
        let result = get_eos_action_name_from_serialized_action(
            &serialized_action
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_address_serialized_action() {
        let expected_result = "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
            .to_string();
        let serialized_action = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .serialized_action
            .clone();
        let result = get_redeem_address_from_serialized_action(
            &serialized_action
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_params_from_action_data_1() {
        let expected_result = RedeemParams {
            amount: 5111,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
                .to_string(),
            from: EosAccountName::from_str("provtestable")
                .unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
            &"aebe7cd1a4687485bc5db87bfb1bdfb44bd1b7f9c080e5cb178a411fd99d2fd5"
                .to_string()
            ).unwrap(),
        };
        let action_data = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .clone();
        let result = RedeemParams::from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }
}
