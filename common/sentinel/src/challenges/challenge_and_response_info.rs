use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use super::{Challenge, ChallengeResponseSignatureInfo};

#[derive(Clone, Debug, Deref, Constructor, Deserialize, Serialize)]
pub struct ChallengeAndResponseInfos(Vec<ChallengeAndResponseInfo>);

#[derive(Clone, Debug, Getters, Constructor, Deserialize, Serialize)]
pub struct ChallengeAndResponseInfo {
    challenge: Challenge,
    response_info: ChallengeResponseSignatureInfo,
}
