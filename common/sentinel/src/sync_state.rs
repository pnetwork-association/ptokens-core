use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use crate::NetworkId;

#[derive(Clone, Debug, Serialize, Deserialize, Deref, Constructor)]
pub struct SyncState(Vec<SyncStatus>);

impl From<(Vec<NetworkId>, Vec<u64>, Vec<u64>)> for SyncState {
    fn from((ids, core_block_nums, rpc_block_nums): (Vec<NetworkId>, Vec<u64>, Vec<u64>)) -> Self {
        // NOTE: We just use defaults if incorrect block numbers are supplied in the vecs.
        Self::new(
            ids.into_iter()
                .enumerate()
                .map(|(i, id)| {
                    SyncStatus::new(
                        id,
                        *core_block_nums.get(i).unwrap_or(&0),
                        *rpc_block_nums.get(i).unwrap_or(&0),
                    )
                })
                .collect::<Vec<SyncStatus>>(),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    network_id: NetworkId,
    core_latest_block_num: u64,
    node_latest_block_num: u64,
    delta: u64,
}

impl SyncStatus {
    pub fn new(network_id: NetworkId, core_latest_block_num: u64, node_latest_block_num: u64) -> Self {
        Self {
            network_id,
            core_latest_block_num,
            node_latest_block_num,
            delta: if node_latest_block_num > core_latest_block_num {
                node_latest_block_num - core_latest_block_num
            } else {
                0
            },
        }
    }
}
