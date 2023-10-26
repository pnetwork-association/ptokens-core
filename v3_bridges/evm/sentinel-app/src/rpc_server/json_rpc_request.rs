use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use super::type_aliases::{RpcId, RpcParams};

#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct JsonRpcRequest {
    id: RpcId,
    method: String,
    params: RpcParams,
}
