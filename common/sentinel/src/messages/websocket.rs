use base64::{engine::general_purpose, Engine};
use common_metadata::MetadataChainId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Responder, SentinelError};

#[derive(Error, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Error {
    #[error("core not initialized for chain id: {0}")]
    Uninitialized(MetadataChainId),

    #[error("core already initialized for chain id: {0}")]
    AlreadyInitialized(MetadataChainId),
}

pub type Confirmations = u64;

#[derive(Debug)]
pub struct WebSocketMessages(
    pub WebSocketMessagesEncodable,
    pub Responder<WebSocketMessagesEncodable>,
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodable {
    Error(Error),
    Success(String),
    GetLatestBlockNum(MetadataChainId),
    Initialize {
        host_id: MetadataChainId,
        host_confs: Confirmations,
        native_id: MetadataChainId,
        native_confs: Confirmations,
    },
}

impl TryFrom<String> for WebSocketMessagesEncodable {
    type Error = SentinelError;

    fn try_from(s: String) -> Result<Self, SentinelError> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<&str> for WebSocketMessagesEncodable {
    type Error = SentinelError;

    fn try_from(s: &str) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(&general_purpose::STANDARD_NO_PAD.decode(s)?)?)
    }
}

impl TryInto<String> for WebSocketMessagesEncodable {
    type Error = SentinelError;

    fn try_into(self) -> Result<String, SentinelError> {
        Ok(general_purpose::STANDARD_NO_PAD.encode(serde_json::to_vec(&self)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_messages_encodable_should_make_serde_roundtrip() {
        let id = MetadataChainId::EthereumMainnet;
        let m = WebSocketMessagesEncodable::GetLatestBlockNum(id);
        let s: String = m.clone().try_into().unwrap();
        let expected_s = "eyJHZXRMYXRlc3RCbG9ja051bSI6IkV0aGVyZXVtTWFpbm5ldCJ9";
        assert_eq!(s, expected_s);
        let r = WebSocketMessagesEncodable::try_from(s).unwrap();
        assert_eq!(r, m);
    }
}
