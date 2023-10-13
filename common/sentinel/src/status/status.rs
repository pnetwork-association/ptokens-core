use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use common_eth::{Chain, ChainBlockData, EthPrivateKey, EthSignature, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use rbtag::{BuildDateTime, BuildInfo};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use thiserror::Error;

use crate::{get_utc_timestamp, NetworkId, SentinelError, WebSocketMessagesEncodable};

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

pub const MIN_STATUS_PUBLISHING_FREQENCY: u64 = 15;
pub const MAX_STATUS_PUBLISHING_FREQENCY: u64 = 60 * 10;

#[derive(BuildDateTime, BuildInfo)]
struct GitCommitHashStruct;

#[derive(Debug, Error)]
pub enum SentinelStatusError {
    #[error("cannot sign status, signature already present")]
    SignatureAlreadyPresent,

    #[error("could not sign `SentinelStatus`: {0}")]
    SigningError(String),

    #[error("sentinel status serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("invalid publishing frequency {0} - must be between {MIN_STATUS_PUBLISHING_FREQENCY} & {MAX_STATUS_PUBLISHING_FREQENCY}")]
    InvalidPublishingFrequency(u64),

    #[error("cannot convert from: '{from}' to: 'SentinelStatus'")]
    CannotConvert { from: String },

    #[error("no mcid in sync status: {0}")]
    NoMcid(SyncStatus),
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    latest_block_hash: String,
    latest_block_number: u64,
    latest_block_timestamp: u64,

    #[getter(skip)]
    #[serde(skip_serializing)] // NOTE: This is used to generate the string key of the hashmap
    mcid: Option<MetadataChainId>,
}

impl fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", json!(self))
    }
}

impl SyncStatus {
    fn mcid(&self) -> Result<MetadataChainId, SentinelStatusError> {
        self.mcid.ok_or_else(|| SentinelStatusError::NoMcid(self.clone()))
    }
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
            mcid: Some(*c.chain_id()),
            latest_block_number: c.latest_block_num(),
            latest_block_timestamp: c.latest_block_timestamp().as_secs(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentinelStatus {
    actor_type: String,
    signer_address: String,
    #[serde(serialize_with = "ordered_map")]
    software_versions: HashMap<String, String>,
    #[serde(serialize_with = "ordered_map")]
    sync_state: HashMap<String, SyncStatus>,
    timestamp: u64,
    version: u8,
    // NOTE: We serialize the struct, sign it, then add the signature to is.
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
}

// NOTE: We require the keys of the json to be in alphabetical order, including any and all keys in
// hashmaps.
fn ordered_map<S: Serializer, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

impl TryFrom<WebSocketMessagesEncodable> for SentinelStatus {
    type Error = SentinelStatusError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        match m {
            WebSocketMessagesEncodable::Success(json) => Ok(serde_json::from_value(json)?),
            _ => Err(SentinelStatusError::CannotConvert { from: m.to_string() }),
        }
    }
}

impl SentinelStatus {
    fn init(signer: EthAddress, git_commit_hash: String, chains: Vec<Chain>) -> Result<Self, SentinelError> {
        let version = 0;
        let signature = None;
        let timestamp = get_utc_timestamp()?;
        let actor_type = "sentinel".to_string();
        let signer_address = format!("0x{}", hex::encode(signer.as_bytes()));

        let mut sync_state = HashMap::<String, SyncStatus>::new();
        for chain in chains {
            let sync_status = SyncStatus::from(chain);
            let key = NetworkId::try_from(sync_status.mcid()?)?.to_hex()?;
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
            version,
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
            // NOTE ethers js is used in other v3 pToken components, which internally stringifies a json
            // and signs those utf8 bytes. (With the eth signature prefix, which the signing fxn adds
            // for us)
            pk.hash_and_sign_msg_with_eth_prefix(self.to_string().as_bytes())
                .map_err(|e| SentinelStatusError::SigningError(format!("{e}")))
        }
    }

    pub fn new(pk: &EthPrivateKey, chains: Vec<Chain>) -> Result<Self, SentinelError> {
        let git_commit_hash = GitCommitHashStruct {}.get_build_commit_long().to_string();
        let signer = pk.to_address();
        let mut status = Self::init(signer, git_commit_hash, chains)?;
        let sig = status.sign(pk)?;
        status.add_signature(sig);
        Ok(status)
    }
}

impl fmt::Display for SentinelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", json!(self))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn should_correctly_sign_status_json() {
        let git_commit_hash = "some static string so we can assert the signature".to_string();
        let pk = EthPrivateKey::from_str("aacb9c865008b5a8b7666a57f5d609347d3f311fd7b98e1d52603ed9a58876c9").unwrap();
        let chains = vec![];
        let mut status = SentinelStatus::init(pk.to_address(), git_commit_hash, chains).unwrap();
        status.timestamp = 1337; // NOTE: So the signature becomes deterministic
        let sig = status.sign(&pk).unwrap();
        status.add_signature(sig);
        let expected_signature =
        Some("0x948dbe1402d3f8fd54cc9c1a117663ec8d955081e6cfe1000a500649e7f8da813884b741bc247459279da9775158130a4601fe94d54768777f388912e4991b6d1c".to_string());
        assert_eq!(status.signature, expected_signature);
    }

    #[test]
    fn should_match_javascript_signatures_made_by_guardians() {
        // NOTE: See reference json at top of file

        let sync_state_struct = SyncStatus {
            mcid: Some(MetadataChainId::XDaiMainnet),
            latest_block_hash: "0x81b4c556ffb342d579cb3feadcdfe2440d62c5f7c6300ed1635bca347dd34f39".to_string(),
            latest_block_number: 30031338,
            latest_block_timestamp: 1695029854,
        };
        let mut sync_state = HashMap::<String, SyncStatus>::new();
        let key = NetworkId::try_from(sync_state_struct.mcid().unwrap())
            .unwrap()
            .to_hex()
            .unwrap();
        sync_state.insert(key, sync_state_struct);
        let mut software_versions = HashMap::<String, String>::new();
        software_versions.insert("listener".to_string(), "1.0.0".to_string());
        software_versions.insert("processor".to_string(), "1.0.0".to_string());

        let status = SentinelStatus {
            actor_type: "guardian".to_string(),
            signer_address: "0x89E8cf56bc3B6C492098e46Da2686c9B5D56951f".to_string(),
            software_versions,
            sync_state,
            timestamp: 1695027477,
            version: 0,
            signature: None,
        };
        // NOTE: The example has multiple elements in its software versions, so we assert this
        // pre-image to ensure they're serialized in the alphabetical order we expect, instead of
        // the randomized order the hashmap would other default to.
        let expected_serialization = "7b226163746f7254797065223a22677561726469616e222c227369676e657241646472657373223a22307838394538636635366263334236433439323039386534364461323638366339423544353639353166222c22736f66747761726556657273696f6e73223a7b226c697374656e6572223a22312e302e30222c2270726f636573736f72223a22312e302e30227d2c2273796e635374617465223a7b2230786434316231633562223a7b226c6174657374426c6f636b48617368223a22307838316234633535366666623334326435373963623366656164636466653234343064363263356637633633303065643136333562636133343764643334663339222c226c6174657374426c6f636b4e756d626572223a33303033313333382c226c6174657374426c6f636b54696d657374616d70223a313639353032393835347d7d2c2274696d657374616d70223a313639353032373437372c2276657273696f6e223a307d";
        let serialized = hex::encode(status.to_string().as_bytes());
        assert_eq!(serialized, expected_serialization);
        let pk = EthPrivateKey::from_str("0xff99c246c4c4a585a685c19ab68b711964f3d346cc3b20713af0e025a85f325b").unwrap();
        let sig = status.sign(&pk).unwrap().to_string();
        let expected_sig = "ee4b9501c71d5a4e3a3ce7462ba595f74ace879b9be2992aadee11cbebff6b615223a34c33e3fcde159424e32f5a95d796e3ba1b2dd71109ebe6ad48ecad2d951b";
        assert_eq!(sig, expected_sig)
    }
}
