use common::{dictionaries::eth_evm::EthEvmTokenDictionary, traits::DatabaseInterface, types::Result};
use common_eth::EthState;
use common_fees::DISABLE_FEES;
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    fees_calculator::{FeeCalculator, FeesCalculator},
    int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos},
};

const TX_INFO_TYPE: &str = "IntOnEvmEvmTxInfos";

impl IntOnEvmEvmTxInfo {
    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.native_token_amount = new_amount;
        new_self
    }
}

impl FeeCalculator for IntOnEvmEvmTxInfo {
    fn get_amount(&self) -> U256 {
        debug!(
            "Getting token amount in `{}` of {}",
            TX_INFO_TYPE, self.native_token_amount
        );
        self.native_token_amount
    }

    fn get_token_address(&self) -> EthAddress {
        debug!(
            "Getting token address in `{}` of {}",
            TX_INFO_TYPE, self.eth_token_address
        );
        self.eth_token_address
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.native_token_amount {
            Err(format!("Cannot subtract amount from `{}`: subtrahend too large!", TX_INFO_TYPE).into())
        } else {
            let new_amount = self.native_token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `{}`!",
                subtrahend, self.native_token_amount, new_amount, TX_INFO_TYPE,
            );
            Ok(self.update_amount(new_amount))
        }
    }
}

impl FeesCalculator for IntOnEvmEvmTxInfos {
    fn get_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `{}`...", TX_INFO_TYPE);
        self.iter()
            .map(|info| info.calculate_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if *fee == U256::zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<IntOnEvmEvmTxInfo>>>()?,
            ))
        })
    }
}

pub fn update_accrued_fees_in_dictionary_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `{}`!", TX_INFO_TYPE);
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!(
            "✔ No `{}` in state during INT block submission ∴ not taking any fees!",
            TX_INFO_TYPE
        );
        Ok(state)
    } else {
        info!("✔ Accruing fees during INT block submission...");
        let tx_infos = IntOnEvmEvmTxInfos::from_bytes(&state.tx_infos)?;
        EthEvmTokenDictionary::get_from_db(state.db)
            .and_then(|dictionary| {
                dictionary.increment_accrued_fees_and_save_in_db(state.db, tx_infos.get_fees(&dictionary)?)
            })
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_evm_tx_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `{}`!", TX_INFO_TYPE);
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!(
            "✔ No `{}` in state during INT block submission ∴ not taking any fees!",
            TX_INFO_TYPE
        );
        Ok(state)
    } else {
        info!(
            "✔ Accounting for fees in `{}` during INT block submission...",
            TX_INFO_TYPE
        );
        let tx_infos = IntOnEvmEvmTxInfos::from_bytes(&state.tx_infos)?;
        EthEvmTokenDictionary::get_from_db(state.db)
            .and_then(|ref dictionary| tx_infos.subtract_fees(dictionary))
            .and_then(|update_infos| update_infos.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!(
        "✔ Accounting for fees in `{}` during INT block submission...",
        TX_INFO_TYPE
    );
    update_accrued_fees_in_dictionary_and_return_state(state).and_then(account_for_fees_in_evm_tx_infos_in_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_evm_tx_info;

    #[test]
    fn should_calculate_eth_on_evm_evm_tx_info_fee() {
        let info = get_sample_evm_tx_info();
        let fee_basis_points = 25;
        let result = info.calculate_fee(fee_basis_points);
        let expected_result = U256::from_dec_str("3").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_amount_from_eth_on_evm_evm_tx_info() {
        let info = get_sample_evm_tx_info();
        let subtrahend = U256::from(337);
        let result = info.subtract_amount(subtrahend).unwrap();
        let expected_native_token_amount = U256::from_dec_str("1000").unwrap();
        assert_eq!(result.native_token_amount, expected_native_token_amount)
    }

    #[test]
    fn should_fail_to_subtract_too_large_amount_from_eth_on_evm_evm_tx_info() {
        let info = get_sample_evm_tx_info();
        let subtrahend = info.native_token_amount + 1;
        let result = info.subtract_amount(subtrahend);
        assert!(result.is_err());
    }
}
