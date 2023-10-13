use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref};

use super::{Challenge, ChallengesError};

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref)]
pub struct Challenges(Vec<Challenge>);

impl Challenges {
    pub fn from_sub_mat(sub_mat: &EthSubmissionMaterial) -> Result<Self, ChallengesError> {
        todo!("this");
    }
}
