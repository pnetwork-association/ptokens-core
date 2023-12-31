use common::{traits::DatabaseInterface, types::Result};
use common_eth::EthState;
use common_safe_addresses::SAFE_ETH_ADDRESS;
use ethereum_types::Address as EthAddress;

use crate::evm::eth_tx_info::{Erc20OnEvmEthTxInfo, Erc20OnEvmEthTxInfos};

create_safe_address_diversion_fxns_v2!(
    "Erc20OnEvmEthTxInfo" => EthState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
