use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use rust_algorand::{AlgorandAddress, AlgorandAppId};

use crate::{metadata::metadata_chain_id::MetadataChainId, types::Bytes};

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
pub struct IntOnAlgoAlgoTxInfo {
    pub user_data: Bytes,
    pub algo_asset_id: u64,
    pub host_token_amount: U256,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_address: AlgorandAddress,
    pub destination_chain_id: MetadataChainId,
    pub issuance_manager_app_id: AlgorandAppId,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct IntOnAlgoAlgoTxInfos(pub Vec<IntOnAlgoAlgoTxInfo>);
