use derive_more::{Constructor, Deref};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{UserOp, UserOpFlag};
use crate::SentinelError;

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Constructor)]
pub struct UserOpListEntry {
    uid: EthHash,
    flag: UserOpFlag,
}

impl TryFrom<&UserOp> for UserOpListEntry {
    type Error = SentinelError;

    fn try_from(o: &UserOp) -> Result<Self, Self::Error> {
        Ok(Self::new(o.to_uid()?, o.to_flag()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Deref)]
pub struct UserOpList(Vec<UserOpListEntry>);
