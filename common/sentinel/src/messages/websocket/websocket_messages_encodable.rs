use std::{fmt, str::FromStr};

use base64::{engine::general_purpose, Engine};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::WebSocketMessagesEncodableDbOps;
use crate::{
    DebugSignature,
    NetworkId,
    SentinelError,
    UserOpUniqueId,
    WebSocketMessagesCancelUserOpArgs,
    WebSocketMessagesError,
    WebSocketMessagesGetCancellableUserOpArgs,
    WebSocketMessagesInitArgs,
    WebSocketMessagesProcessBatchArgs,
    WebSocketMessagesResetChainArgs,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesEncodable {
    Null,
    GetUserOps,
    Success(Json),
    GetUserOpList,
    GetChallengesList,
    GetInclusionProof,
    GetChallenge(EthHash),
    GetUnsolvedChallenges,
    RemoveChallenge(EthHash),
    GetStatus(Vec<NetworkId>),
    GetUserOp(UserOpUniqueId),
    GetCoreState(Vec<NetworkId>),
    RemoveUserOp(UserOpUniqueId),
    Error(WebSocketMessagesError),
    SetChallengesToSolved(Vec<EthHash>),
    GetLatestBlockInfos(Vec<NetworkId>),
    DbOps(WebSocketMessagesEncodableDbOps),
    GetRegistrationSignature(EthAddress, u64),
    Initialize(Box<WebSocketMessagesInitArgs>),
    ResetChain(Box<WebSocketMessagesResetChainArgs>),
    ProcessBatch(Box<WebSocketMessagesProcessBatchArgs>),
    AddDebugSigners(Vec<(String, EthAddress)>, DebugSignature),
    GetCancellableUserOps(Box<WebSocketMessagesGetCancellableUserOpArgs>),
    GetUserOpCancellationSignature(Box<WebSocketMessagesCancelUserOpArgs>),
}

impl TryFrom<WebSocketMessagesEncodable> for Json {
    type Error = WebSocketMessagesError;

    fn try_from(w: WebSocketMessagesEncodable) -> Result<Json, Self::Error> {
        match w {
            WebSocketMessagesEncodable::Success(json) => Ok(json),
            other => Err(WebSocketMessagesError::CannotConvert {
                to: "json".to_string(),
                from: other.to_string(),
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
            Self::Null => "Null".to_string(),
            Self::DbOps(op) => format!("{op}"),
            Self::Error(e) => format!("Error: {e}"),
            Self::Success(_) => "Success".to_string(),
            Self::GetUserOps => "GetUserOps".to_string(),
            Self::GetUserOp(_) => "GetUserOp".to_string(),
            Self::GetStatus(..) => "GetStatus".to_string(),
            Self::Initialize(_) => "Initialize".to_string(),
            Self::ResetChain(_) => "ResetChain".to_string(),
            Self::GetUserOpList => "GetUserOpList".to_string(),
            Self::RemoveUserOp(_) => "RemoveUserOp".to_string(),
            Self::GetCoreState(..) => "GetCoreState".to_string(),
            Self::ProcessBatch(..) => "ProcessBatch".to_string(),
            Self::GetChallenge(..) => "GetChallenge".to_string(),
            Self::AddDebugSigners(..) => "AddDebugSigners".to_string(),
            Self::GetChallengesList => "GetChallengesList".to_string(),
            Self::RemoveChallenge(..) => "RemoveChallenge".to_string(),
            Self::GetInclusionProof => "GetInclusionProof".to_string(),
            Self::GetLatestBlockInfos(..) => "GetLatestBlockInfos".to_string(),
            Self::GetUnsolvedChallenges => "GetUnsolvedChallenges".to_string(),
            Self::GetCancellableUserOps(_) => "GetCancellableUserOps".to_string(),
            Self::SetChallengesToSolved(..) => "SetChallengesToSolved".to_string(),
            Self::GetRegistrationSignature(..) => "GetRegistrationSignature".to_string(),
            Self::GetUserOpCancellationSignature(..) => "GetUserOpCancellationSignature".to_string(),
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
            "init" | "initialize" => Ok(Self::Initialize(Box::new(WebSocketMessagesInitArgs::try_from(
                args[1..].to_vec(),
            )?))),
            "getCancellableUserOps" => Ok(Self::GetCancellableUserOps(Box::new(
                WebSocketMessagesGetCancellableUserOpArgs::try_from(args[1..].to_vec())?,
            ))),
            "reset" | "resetChain" => Ok(Self::ResetChain(Box::new(WebSocketMessagesResetChainArgs::try_from(
                args[1..].to_vec(),
            )?))),
            "removeUserOp" => {
                let uid = UserOpUniqueId::from_str(&args[1])?;
                Ok(Self::RemoveUserOp(uid))
            },
            "get" | "put" | "delete" => Ok(Self::DbOps(WebSocketMessagesEncodableDbOps::try_from(args)?)),
            _ => {
                warn!("cannot create WebSocketMessagesEncodable from args {args:?}");
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
