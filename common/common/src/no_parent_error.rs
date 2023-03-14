#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, derive_more::Constructor)]
pub struct NoParentError {
    pub block_num: u64,
    pub message: String,
    pub bridge_side: crate::BridgeSide,
}

impl NoParentError {
    pub fn to_json(&self) -> crate::types::Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl std::fmt::Display for NoParentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.to_json() {
            Ok(j) => write!(f, "{j}"),
            Err(e) => write!(f, "error parsing no parent error to json: {e}"),
        }
    }
}
