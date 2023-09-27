use std::{collections::HashMap, env};

use common_eth::{Chain, ChainBlockData, EthPrivateKey, EthSignature, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/* JSON Reference:
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

#[derive(Debug, Error)]
pub enum SentinelStatusError {
    #[error("cannot sign status, signature already present")]
    SignatureAlreadyPresent,

    #[error("could not sign `SentinelStatus`: {0}")]
    SigningError(String),

    #[error("sentinel status serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

use crate::{get_utc_timestamp, NetworkId, SentinelError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    #[serde(skip_serializing)] // NOTE: This is used to generate the string key of the hashmap
    mcid: MetadataChainId,
    latest_block_number: u64,
    latest_block_hash: String,
    latest_block_timestamp: u64,
}

impl From<Chain> for SyncStatus {
    fn from(c: Chain) -> Self {
        SyncStatus::from(&c)
    }
}

impl From<&Chain> for SyncStatus {
    fn from(c: &Chain) -> Self {
        // NOTE: Due to forks, there's always the possibility of > 1 block at any point in the chain
        let latest_block_data: Vec<ChainBlockData> =
            c.get_latest_block_data().map(|x| x.to_vec()).unwrap_or_else(Vec::new);
        let h = if latest_block_data.is_empty() {
            EthHash::zero()
        } else {
            *latest_block_data[0].hash()
        };
        let latest_block_hash = format!("0x{}", hex::encode(h.as_bytes()));

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
pub struct SentinelStatus {
    timestamp: u64,
    actor_type: String,
    signer_address: String,
    sync_state: HashMap<String, SyncStatus>,
    software_versions: HashMap<String, String>,
    // NOTE: We serialize the struct, sign it, then add the signature to is.
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
}

impl SentinelStatus {
    fn init(signer: EthAddress, git_commit_hash: String, chains: Vec<Chain>) -> Result<Self, SentinelError> {
        let signature = None;
        let timestamp = get_utc_timestamp()?;
        let actor_type = "sentinel".to_string();
        let signer_address = format!("0x{}", hex::encode(signer.as_bytes()));

        let mut sync_state = HashMap::<String, SyncStatus>::new();
        for chain in chains {
            let sync_status = SyncStatus::from(chain);
            let key = NetworkId::try_from(sync_status.mcid())?.to_string();
            sync_state.insert(key, sync_status);
        }

        let mut software_versions = HashMap::<String, String>::new();
        software_versions.insert(actor_type.clone(), git_commit_hash);

        Ok(Self {
            sync_state,
            timestamp,
            actor_type,
            signer_address,
            signature,
            software_versions,
        })
    }

    fn add_signature(&mut self, sig: EthSignature) {
        // TODO Validate the signer matches the signature?
        if self.signature.is_none() {
            self.signature = Some(format!("0x{sig}"))
        }
    }

    fn sign(&self, pk: &EthPrivateKey) -> Result<EthSignature, SentinelStatusError> {
        if self.signature.is_some() {
            Err(SentinelStatusError::SignatureAlreadyPresent)
        } else {
            // NOTE ethers js is used in other components, which internally stringifies a json and
            // signs those utf8 bytes. (With the eth signature prefix, which the signing fxn adds
            // for us)
            pk.sign_eth_prefixed_msg_bytes(serde_json::to_string(self)?.as_bytes())
                .map_err(|e| SentinelStatusError::SigningError(format!("{e}")))
        }
    }

    pub fn new(pk: &EthPrivateKey, chains: Vec<Chain>) -> Result<Self, SentinelError> {
        let git_commit_hash =
            env::var("GIT_HASH").unwrap_or("`GIT_HASH` env variable was not set at build time".to_string());
        let signer = pk.to_address();
        let mut status = Self::init(signer, git_commit_hash, chains)?;
        let sig = status.sign(pk)?;
        status.add_signature(sig);
        Ok(status)
    }
}
