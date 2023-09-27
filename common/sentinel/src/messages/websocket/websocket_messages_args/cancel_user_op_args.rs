use common_metadata::MetadataChainId;
use derive_getters::{Dissolve, Getters};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::UserOp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Dissolve, Constructor)]
pub struct WebSocketMessagesCancelUserOpArgs {
    mcids: Vec<MetadataChainId>,
    op: UserOp,
}
