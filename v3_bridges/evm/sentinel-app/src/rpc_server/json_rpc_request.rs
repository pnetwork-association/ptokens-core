use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use super::type_aliases::{RpcId, RpcParams};

#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct JsonRpcRequest {
    id: RpcId,
    #[getter(skip)]
    method: String,
    #[getter(skip)]
    params: RpcParams,
}

impl JsonRpcRequest {
    pub(super) fn params(&self) -> RpcParams {
        self.params.clone()
    }

    pub(super) fn method(&self) -> String {
        self.method.clone()
    }
}
