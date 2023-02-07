use common::{safe_addresses::SAFE_ETH_ADDRESS, state::EthState, traits::DatabaseInterface, types::Result};
use ethereum_types::Address as EthAddress;

use crate::eth::evm_tx_info::{Erc20OnEvmEvmTxInfo, Erc20OnEvmEvmTxInfos};

create_safe_address_diversion_fxns_v2!(
    "Erc20OnEvmEvmTxInfo" => EthState => "evm" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
