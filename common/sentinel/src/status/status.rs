use std::collections::HashMap;

use common_eth::{Chain, ChainBlockData};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use crate::{SentinelError, NetworkId};

/* Reference:
{
  actorType: 'guardian',
  signerAddress: '0xdB30d31Ce9A22f36a44993B1079aD2D201e11788',
  softwareVersions: { listener: '1.0.0', processor: '1.0.0' },
  timestamp: 1695741106,
  syncState: {
    '0xf9b459a1': {
      latestBlockHash: '0x1419611597cb87a626980a0fa74fa5651f7396d1b6f16c246463e25d618ba868',
      latestBlockNumber: 48010525,
      latestBlockTimestamp: 1695741105
    }
  },
  signature: '0xd55c83bbf7b8b43c2356885806d07e8e65f6096724e253f8794b82df5f8d266a26c8643074ccaed09e1c5f1f73231e1dbed28f289785b1004738c65e2f9b61951c'
}
 */

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    #[serde(skip_serializing)] // NOTE: This is used to generate the string key of the hashmap
    mcid: MetadataChainId,
    latest_block_hash: EthHash,
    latest_block_number: u64,
    latest_block_timestamp: u64,
}

impl From<&Chain> for SyncStatus {
    fn from(c: &Chain) -> Self {
        // NOTE: Due to forks, there's always the possibility of > 1 block at any point in the chain
        let latest_block_data: Vec<ChainBlockData> =
            c.get_latest_block_data().map(|x| x.to_vec()).unwrap_or_else(Vec::new);
        let latest_block_hash = if latest_block_data.is_empty() {
            EthHash::zero()
        } else {
            *latest_block_data[0].hash()
        };

        Self {
            latest_block_hash,
            mcid: *c.chain_id(),
            latest_block_number: c.latest_block_num(),
            latest_block_timestamp: c.latest_block_timestamp().as_secs(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    sync_state: HashMap<String, SyncStatus>,
}

/*
impl Status {
    fn new() -> Result<Self, SentinelError> {

    }
}
*/



/*
impl Status {
    fn new(sync_statuses: Vec<SyncStatus>) -> Self {
        let mut map = HashMap::<String, SyncStatus>::new();
        sync_statuses
            .iter()
            .for_each(|s| {
                todo!("make this a PREFIXED hex string");
                let k = MetadataChainId::to_network_id_bytes().to_string();
                map.insert(k, s);
            });
    }
}
*/
