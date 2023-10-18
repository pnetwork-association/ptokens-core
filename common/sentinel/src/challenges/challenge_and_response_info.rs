use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::{Challenge, ChallengeResponseSignatureInfo, ChallengesError};
use crate::WebSocketMessagesEncodable;

#[derive(Clone, Debug, Deref, Constructor, Deserialize, Serialize)]
pub struct ChallengeAndResponseInfos(Vec<ChallengeAndResponseInfo>);

impl ChallengeAndResponseInfos {
    pub fn get_ids(&self) -> Result<Vec<EthHash>, ChallengesError> {
        self.iter().map(|x| x.challenge().id()).collect()
    }
}

#[derive(Clone, Debug, Getters, Constructor, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeAndResponseInfo {
    challenge: Challenge,
    response_info: ChallengeResponseSignatureInfo,
}

impl TryFrom<WebSocketMessagesEncodable> for ChallengeAndResponseInfos {
    type Error = ChallengesError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}
