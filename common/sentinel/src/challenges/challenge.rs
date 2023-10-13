use std::str::FromStr;

use common_eth::{EthLog, EthLogExt, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use super::ChallengesError;
use crate::{Actor, ActorType, NetworkId};

lazy_static! {
    pub(super) static ref CHALLENGE_PENDING_EVENT_TOPIC: EthHash = EthHash::from_str("").expect("this not to fail");
}

#[derive(Debug, Clone, Eq, PartialEq, Getters, Constructor)]
pub struct Challenge {
    nonce: U256,
    actor: Actor,
    timestamp: u64,
    mcid: MetadataChainId,
}

impl Challenge {
    pub fn from_sub_mat(sub_mat: &EthSubmissionMaterial, pnetwork_hub: &EthAddress) -> Result<Self, ChallengesError> {
        todo!("this = don't forget to filter addresses");
    }
}
