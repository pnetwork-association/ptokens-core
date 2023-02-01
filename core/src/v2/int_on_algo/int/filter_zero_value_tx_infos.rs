use ethereum_types::U256;

use crate::{
    int_on_algo::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnAlgoAlgoTxInfos {
    fn get_host_token_amounts(&self) -> Vec<U256> {
        self.iter()
            .map(|tx_info| tx_info.host_token_amount)
            .collect::<Vec<U256>>()
    }

    pub fn filter_out_zero_values(&self) -> Result<Self> {
        let host_token_amounts = self.get_host_token_amounts();
        Ok(Self::new(
            self.iter()
                .zip(host_token_amounts.iter())
                .filter(
                    |(tx_info, host_token_amount)| match *host_token_amount != &U256::zero() {
                        true => true,
                        false => {
                            info!(
                                "✘ Filtering out peg in info due to zero ALGO asset amount: {:?}",
                                tx_info
                            );
                            false
                        },
                    },
                )
                .map(|(info, _)| info)
                .cloned()
                .collect::<Vec<IntOnAlgoAlgoTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero value `IntOnAlgoAlgoTxInfos`...");
    let infos = state.int_on_algo_algo_tx_infos.clone();
    debug!("✔ Num `IntOnAlgoAlgoTxInfos` before: {}", infos.len());
    infos.filter_out_zero_values().and_then(|filtered_tx_infos| {
        debug!("✔ Num `IntOnAlgoAlgoTxInfos` after: {}", filtered_tx_infos.len());
        state.replace_int_on_algo_algo_tx_infos(filtered_tx_infos)
    })
}
