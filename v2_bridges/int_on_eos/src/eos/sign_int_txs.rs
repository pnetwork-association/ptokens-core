use common::{traits::DatabaseInterface, types::Result};
use common_chain_ids::EthChainId;
use common_eos::EosState;
use common_eth::{
    encode_erc20_vault_peg_out_fxn_data_with_user_data,
    EthDbUtils,
    EthDbUtilsExt,
    EthPrivateKey,
    EthTransaction,
    EthTransactions,
    ZERO_ETH_VALUE,
};

use crate::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos};

impl IntOnEosIntTxInfos {
    pub fn to_signed_txs(
        &self,
        nonce: u64,
        gas_price: u64,
        chain_id: &EthChainId,
        private_key: &EthPrivateKey,
    ) -> Result<EthTransactions> {
        Ok(EthTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| tx_info.to_signed_tx(nonce + i as u64, gas_price, chain_id, private_key))
                .collect::<Result<Vec<EthTransaction>>>()?,
        ))
    }
}

impl IntOnEosIntTxInfo {
    fn to_signed_tx(
        &self,
        nonce: u64,
        gas_price: u64,
        chain_id: &EthChainId,
        private_key: &EthPrivateKey,
    ) -> Result<EthTransaction> {
        info!(
            "✔ Signing INT tx for amount: {}, to address: {}",
            self.amount, self.destination_address
        );
        encode_erc20_vault_peg_out_fxn_data_with_user_data(
            self.router_address,
            self.int_token_address,
            self.amount,
            self.to_metadata_bytes()?,
        )
        .and_then(|fxn_data| {
            EthTransaction::new_unsigned(
                fxn_data,
                nonce,
                ZERO_ETH_VALUE,
                self.int_vault_address,
                chain_id,
                chain_id.get_erc20_vault_pegout_with_user_data_gas_limit(),
                gas_price,
            )
            .sign(private_key)
        })
    }
}

pub fn maybe_sign_int_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ No tx infos in state ∴ no INT transactions to sign!");
        Ok(state)
    } else {
        info!("✔ Signing INT transactions...");
        IntOnEosIntTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                let eth_db_utils = EthDbUtils::new(state.db);
                tx_infos.to_signed_txs(
                    eth_db_utils.get_eth_account_nonce_from_db()?,
                    eth_db_utils.get_eth_gas_price_from_db()?,
                    &eth_db_utils.get_eth_chain_id_from_db()?,
                    &eth_db_utils.get_eth_private_key_from_db()?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                signed_txs.to_bytes()
            })
            .map(|bytes| state.add_eth_signed_txs(bytes))
    }
}
