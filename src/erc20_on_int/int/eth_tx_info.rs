use std::fmt;

use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    address::Address,
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    },
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::Bytes,
    utils::convert_bytes_to_string,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor)]
pub struct Erc20OnIntEthTxInfo {
    pub user_data: Bytes,
    pub host_token_amount: U256,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub origin_chain_id: EthChainId,
    pub token_recipient: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_vault_address: EthAddress,
    pub eth_token_address: EthAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct Erc20OnIntEthTxInfos(pub Vec<Erc20OnIntEthTxInfo>);

impl fmt::Display for Erc20OnIntEthTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
Erc20OnIntEthTxInfo: {{
    native_token_amount: {},
    token_sender: {},
    originating_tx_hash: {},
    evm_token_address: {},
    eth_token_address: {},
    destination_address: {},
    origin_chain_id: {},
    eth_vault_address: {},
    router_address: {},
    user_data: {},
}}
",
            self.native_token_amount,
            convert_eth_address_to_string(&self.token_sender),
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.evm_token_address),
            convert_eth_address_to_string(&self.eth_token_address),
            &self.destination_address,
            self.origin_chain_id,
            convert_eth_address_to_string(&self.eth_vault_address),
            convert_eth_address_to_string(&self.router_address),
            convert_bytes_to_string(&self.user_data),
        )
    }
}

impl_tx_info_trait!(
    Erc20OnIntEthTxInfo,
    eth_vault_address,
    router_address,
    eth_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);