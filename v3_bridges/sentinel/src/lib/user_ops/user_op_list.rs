use common::Byte;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::UserOpFlags;

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct UserOpList(Vec<(EthHash, UserOpFlags)>);
