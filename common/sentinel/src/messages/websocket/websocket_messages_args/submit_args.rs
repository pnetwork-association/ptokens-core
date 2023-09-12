use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::EthSubmissionMaterial;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct WebSocketMessagesSubmitArgs {
    dry_run: bool,
    validate: bool,
    reprocess: bool,
    side: BridgeSide,
    eth_chain_id: EthChainId,
    pnetwork_hub: EthAddress,
    sub_mat: EthSubmissionMaterial,
}