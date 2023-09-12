use std::{fmt, str::FromStr};

use base64::{engine::general_purpose, Engine};
use common::{AppError as CommonError, BridgeSide};
use common_chain_ids::EthChainId;
use common_eth::EthSubmissionMaterial;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use thiserror::Error;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{Responder, SentinelError};

#[derive(Error, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesError {
    #[error("core not initialized for chain id: {0}")]
    Uninitialized(EthChainId),

    #[error("core already initialized for chain id: {0}")]
    AlreadyInitialized(EthChainId),

    #[error("cannot create websocket message encodable from args: {0:?}")]
    CannotCreate(Vec<String>),

    #[error("cannot create websocket message encodable from {got} args, expected {expected}: {args:?}")]
    NotEnoughArgs {
        got: usize,
        expected: usize,
        args: Vec<String>,
    },

    #[error("could not parse u64 from {0}")]
    ParseInt(String),

    #[error("unrecognized chain id {0}")]
    UnrecognizedEthChainId(String),

    #[error("timed out - strongbox took longer than {0}ms to respond")]
    Timedout(u64),

    #[error("no {side} block found in {struct_name}")]
    NoBlock { side: BridgeSide, struct_name: String },

    #[error("common error: {0}")]
    CommonError(String),

    #[error("sentinel error: {0}")]
    SentinelError(String),

    #[error("java database error: {0}")]
    JavaDb(String),

    #[error("unhandled websocket message: {0}")]
    Unhandled(String),

    #[error("cannot convert from: {from} to: {to}")]
    CannotConvert { from: String, to: String },
}

impl From<CommonError> for WebSocketMessagesError {
    fn from(e: CommonError) -> Self {
        Self::CommonError(format!("{e}"))
    }
}

impl From<SentinelError> for WebSocketMessagesError {
    fn from(e: SentinelError) -> Self {
        Self::SentinelError(format!("{e}"))
    }
}

pub type Confirmations = u64;

#[derive(Debug)]
pub struct WebSocketMessages(
    pub WebSocketMessagesEncodable,
    pub Responder<WebSocketMessagesEncodable>,
);

impl WebSocketMessages {
    pub fn new(msg: WebSocketMessagesEncodable) -> (Self, Receiver<Result<WebSocketMessagesEncodable, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self(msg, tx), rx)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct WebSocketMessagesInitArgs {
    host_validate: bool,
    native_validate: bool,
    host_chain_id: EthChainId,
    host_confirmations: Confirmations,
    native_chain_id: EthChainId,
    native_confirmations: Confirmations,
    host_block: Option<EthSubmissionMaterial>,
    native_block: Option<EthSubmissionMaterial>,
}

#[derive(Debug, Clone, PartialEq, Constructor, Serialize, Deserialize, Getters)]
pub struct WebSocketMessagesSubmitArgs {
    dry_run: bool,
    validate: bool,
    reprocess: bool,
    side: BridgeSide,
    eth_chain_id: EthChainId,
    pnetwork_hub: EthAddress,
    sub_mat: EthSubmissionMaterial,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodable {
    Null,
    GetCoreState,
    Success(Json),
    GetLatestBlockNumbers,
    Error(WebSocketMessagesError),
    Submit(Box<WebSocketMessagesSubmitArgs>),
    Initialize(Box<WebSocketMessagesInitArgs>),
}

impl WebSocketMessagesInitArgs {
    fn name(&self) -> String {
        "WebSocketMessagesInitArgs".into()
    }

    pub fn add_host_block(&mut self, m: EthSubmissionMaterial) {
        self.host_block = Some(m);
    }

    pub fn add_native_block(&mut self, m: EthSubmissionMaterial) {
        self.native_block = Some(m);
    }

    pub fn to_host_sub_mat(&self) -> Result<EthSubmissionMaterial, WebSocketMessagesError> {
        match self.host_block {
            Some(ref b) => Ok(b.clone()),
            None => Err(WebSocketMessagesError::NoBlock {
                side: BridgeSide::Host,
                struct_name: self.name(),
            }),
        }
    }

    pub fn to_native_sub_mat(&self) -> Result<EthSubmissionMaterial, WebSocketMessagesError> {
        match self.native_block {
            Some(ref b) => Ok(b.clone()),
            None => Err(WebSocketMessagesError::NoBlock {
                side: BridgeSide::Native,
                struct_name: self.name(),
            }),
        }
    }
}

impl WebSocketMessagesEncodable {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl fmt::Display for WebSocketMessagesEncodable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "WebSocketMessagesEncodable::";
        let s = match self {
            Self::Null => "Null",
            Self::Error(_) => "Error",
            Self::Submit(..) => "Submit",
            Self::Success(_) => "Success",
            Self::Initialize(_) => "Initialize",
            Self::GetCoreState => "GetCoreState",
            Self::GetLatestBlockNumbers => "GetLatestBlockNumbers",
        };
        write!(f, "{prefix}{s}")
    }
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

impl TryFrom<Vec<&str>> for WebSocketMessagesEncodable {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<&str>) -> Result<Self, WebSocketMessagesError> {
        Self::try_from(args.iter().map(|x| x.to_string()).collect::<Vec<_>>())
    }
}

impl TryFrom<Vec<String>> for WebSocketMessagesEncodable {
    type Error = WebSocketMessagesError;

    fn try_from(args: Vec<String>) -> Result<Self, WebSocketMessagesError> {
        if args.is_empty() {
            return Err(WebSocketMessagesError::CannotCreate(args));
        };
        let cmd = args[0].as_ref();
        match cmd {
            "init" | "initialize" => {
                let expected_num_args = 7;
                if args.len() != expected_num_args {
                    return Err(WebSocketMessagesError::NotEnoughArgs {
                        got: args.len(),
                        expected: expected_num_args,
                        args,
                    });
                }
                Ok(Self::Initialize(Box::new(WebSocketMessagesInitArgs {
                    host_validate: matches!(args[1].as_ref(), "true"),
                    native_validate: matches!(args[2].as_ref(), "true"),
                    host_chain_id: EthChainId::from_str(&args[3])
                        .map_err(|_| WebSocketMessagesError::UnrecognizedEthChainId(args[3].clone()))?,
                    host_confirmations: args[4]
                        .parse::<Confirmations>()
                        .map_err(|_| WebSocketMessagesError::ParseInt(args[4].clone()))?,
                    native_chain_id: EthChainId::from_str(&args[5])
                        .map_err(|_| WebSocketMessagesError::UnrecognizedEthChainId(args[5].clone()))?,
                    native_confirmations: args[6]
                        .parse::<Confirmations>()
                        .map_err(|_| WebSocketMessagesError::ParseInt(args[7].clone()))?,
                    host_block: None,
                    native_block: None,
                })))
            },
            _ => {
                debug!("cannot create WebSocketMessagesEncodable from args {args:?}");
                Err(WebSocketMessagesError::CannotCreate(args))
            },
        }
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
        let m = WebSocketMessagesEncodable::GetCoreState;
        let s: String = m.clone().try_into().unwrap();
        let expected_s = "IkdldEVuY2xhdmVTdGF0ZSI";
        assert_eq!(s, expected_s);
        let r = WebSocketMessagesEncodable::try_from(s).unwrap();
        assert_eq!(r, m);
    }

    #[test]
    fn should_get_init_message_from_string_of_args() {
        let args = vec!["init", "true", "true", "EthereumMainnet", "10", "BscMainnet", "100"];
        let r = WebSocketMessagesEncodable::try_from(args);
        assert!(r.is_ok());
    }
}
