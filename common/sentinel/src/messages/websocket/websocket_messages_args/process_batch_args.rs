use common_eth::EthSubmissionMaterials;
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::NetworkId;

#[derive(Debug, Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Dissolve)]
pub struct WebSocketMessagesProcessBatchArgs {
    validate: bool,
    dry_run: bool,
    reprocess: bool,
    network_id: NetworkId,
    pnetwork_hub: EthAddress,
    sub_mat_batch: EthSubmissionMaterials,
    governance_address: Option<EthAddress>,
}

impl WebSocketMessagesProcessBatchArgs {
    pub fn new_for_syncer(
        validate: bool,
        network_id: NetworkId,
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
            network_id,
            pnetwork_hub,
            sub_mat_batch,
            governance_address,
        ))
    }
}
