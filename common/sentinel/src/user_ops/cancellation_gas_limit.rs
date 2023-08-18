use common_chain_ids::EthChainId;

use super::{UserOp, USER_OP_CANCEL_TX_GAS_LIMIT};

impl UserOp {
    pub fn cancellation_gas_limit(chain_id: &EthChainId) -> usize {
        match chain_id {
            EthChainId::XDaiMainnet => 200_000,
            EthChainId::ArbitrumMainnet => 2_000_000,
            _ => USER_OP_CANCEL_TX_GAS_LIMIT as usize,
        }
    }
}
