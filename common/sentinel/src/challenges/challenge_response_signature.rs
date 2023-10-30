use std::fmt;

use common::Bytes;
use common_eth::{EthSignature, ETH_SIGNATURE_NUM_BYTES};
use derive_getters::{Dissolve, Getters};
use derive_more::{Constructor, Deref};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{ActorInclusionProof, ChallengesError, SentinelError, WebSocketMessagesEncodable};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref)]
pub struct ChallengeResponseSignature(Bytes);

impl ChallengeResponseSignature {
    pub fn new(bs: Bytes) -> Result<Self, ChallengesError> {
        let name = "ChallengeResponseSignatureInfo";
        debug!("creating `{name}` struct...");
        let l = bs.len();
        if l != ETH_SIGNATURE_NUM_BYTES {
            Err(ChallengesError::NotEnoughBytes {
                got: l,
                expected: format!("{ETH_SIGNATURE_NUM_BYTES}"),
                location: name.to_string(),
            })
        } else {
            Ok(Self(bs))
        }
    }
}

impl From<ChallengeResponseSignature> for EthAbiToken {
    fn from(val: ChallengeResponseSignature) -> Self {
        EthAbiToken::Bytes(val.to_vec())
    }
}

impl fmt::Display for ChallengeResponseSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

#[derive(Clone, Debug, Constructor, Serialize, Deserialize, Deref)]
pub struct ChallengeResponseSignatureInfos(Vec<ChallengeResponseSignatureInfo>);

#[derive(Clone, Debug, Default, Constructor, Serialize, Deserialize, Getters, Dissolve)]
pub struct ChallengeResponseSignatureInfo {
    id: EthHash,
    signer: EthAddress,
    proof: ActorInclusionProof,
    sig: ChallengeResponseSignature,
}

impl fmt::Display for ChallengeResponseSignatureInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "   sig: 0x{}", self.sig)?;
        write!(f, "    id: 0x{}", self.id)?;
        write!(f, "signer: 0x{}", hex::encode(self.signer.as_bytes()))?;
        write!(f, "   proof: {}", self.proof)
    }
}

impl TryFrom<WebSocketMessagesEncodable> for ChallengeResponseSignatureInfo {
    type Error = SentinelError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<ChallengeResponseSignatureInfo, Self::Error> {
        debug!("trying to get `ChallengeResponseSignatureInfo` from `WebSocketMessagesEncodable`...");
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}

impl From<EthSignature> for ChallengeResponseSignature {
    fn from(es: EthSignature) -> ChallengeResponseSignature {
        ChallengeResponseSignature(es.0.to_vec())
    }
}
