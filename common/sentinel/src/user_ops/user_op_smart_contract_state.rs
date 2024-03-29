use std::fmt;

use common::{Byte, Bytes};
use common_eth::encode_fxn_call;
use ethereum_types::U256;

use super::{UserOp, UserOpError, UserOpState};
use crate::SentinelError;

const GET_USER_OP_STATE_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"userDataProtocolFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"networkFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"forwardNetworkFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"forwardDestinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"originAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"},{\"internalType\":\"bool\",\"name\":\"isForProtocol\",\"type\":\"bool\"}],\"internalType\":\"struct IPNetworkHub.Operation\",\"name\":\"operation\",\"type\":\"tuple\"}],\"name\":\"operationStatusOf\",\"outputs\":[{\"internalType\":\"enum IPNetworkHub.OperationStatus\",\"name\":\"\",\"type\":\"uint8\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserOpSmartContractState {
    Null      = 0,
    Enqueued  = 1,
    Executed  = 2,
    Cancelled = 3,
}

impl TryFrom<UserOpState> for UserOpSmartContractState {
    type Error = UserOpError;

    fn try_from(s: UserOpState) -> Result<UserOpSmartContractState, Self::Error> {
        match s {
            UserOpState::Enqueued(..) => Ok(UserOpSmartContractState::Enqueued),
            UserOpState::Executed(..) => Ok(UserOpSmartContractState::Executed),
            UserOpState::Cancelled(..) => Ok(UserOpSmartContractState::Cancelled),
            other => Err(UserOpError::CannotDetermineUserOpSmartContractState(other)),
        }
    }
}

impl TryFrom<Byte> for UserOpSmartContractState {
    type Error = UserOpError;

    fn try_from(b: Byte) -> Result<Self, Self::Error> {
        match b {
            0x00 => Ok(Self::Null),
            0x01 => Ok(Self::Enqueued),
            0x02 => Ok(Self::Executed),
            0x03 => Ok(Self::Cancelled),
            _ => Err(UserOpError::UnrecognizedSmartContractUserOpState(b)),
        }
    }
}

impl TryFrom<U256> for UserOpSmartContractState {
    type Error = UserOpError;

    fn try_from(u: U256) -> Result<Self, Self::Error> {
        match u.as_u64() {
            0 => Ok(Self::Null),
            1 => Ok(Self::Enqueued),
            2 => Ok(Self::Executed),
            3 => Ok(Self::Cancelled),
            _ => Err(UserOpError::UnrecognizedUserOpState(u)),
        }
    }
}

impl TryFrom<Bytes> for UserOpSmartContractState {
    type Error = UserOpError;

    fn try_from(bs: Bytes) -> Result<Self, Self::Error> {
        let name = "UserOpSmartContractState";
        debug!("getting '{name}' from bytes...");
        if bs.is_empty() {
            Err(UserOpError::NotEnoughBytes {
                got: 0,
                expected: "some".to_string(),
                location: name.to_string(),
            })
        } else {
            Self::try_from(U256::from_big_endian(&bs))
        }
    }
}

impl fmt::Display for UserOpSmartContractState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Null => "null",
            Self::Enqueued => "enqueued",
            Self::Executed => "executed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

impl Default for UserOpSmartContractState {
    fn default() -> Self {
        Self::Null
    }
}

impl UserOpSmartContractState {
    pub fn is_enqueued(&self) -> bool {
        self == &Self::Enqueued
    }

    pub fn is_cancellable(&self) -> bool {
        self == &Self::Enqueued
    }

    pub fn encode_rpc_call_data(user_op: &UserOp) -> Result<Bytes, SentinelError> {
        let encoded = encode_fxn_call(GET_USER_OP_STATE_ABI, "operationStatusOf", &[
            user_op.to_eth_abi_token()?
        ])?;
        Ok(encoded)
    }
}
