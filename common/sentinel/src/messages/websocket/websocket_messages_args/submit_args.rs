use common::BridgeSide;
use common_eth::EthSubmissionMaterials;
use common_metadata::MetadataChainId;
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Dissolve)]
pub struct WebSocketMessagesSubmitArgs {
    validate: bool,
    dry_run: bool,
    reprocess: bool,
    side: BridgeSide,
    mcid: MetadataChainId,
    pnetwork_hub: EthAddress,
    sub_mat_batch: EthSubmissionMaterials,
    governance_address: Option<EthAddress>,
}

impl WebSocketMessagesSubmitArgs {
    pub fn new_for_syncer(
        validate: bool,
        side: BridgeSide,
        mcid: MetadataChainId,
        pnetwork_hub: EthAddress,
        sub_mat_batch: EthSubmissionMaterials,
        governance_address: Option<EthAddress>,
    ) -> Box<Self> {
        let dry_run = false;
        let reprocess = false;
        Box::new(Self::new(
            validate,
            dry_run,
            reprocess,
            side,
            mcid,
            pnetwork_hub,
            sub_mat_batch,
            governance_address,
        ))
    }
}
