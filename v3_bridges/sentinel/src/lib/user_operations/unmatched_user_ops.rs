use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use super::UserOperations;

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Serialize, Deserialize)]
pub struct UnmatchedUserOps {
    native: UserOperations,
    host: UserOperations,
}
