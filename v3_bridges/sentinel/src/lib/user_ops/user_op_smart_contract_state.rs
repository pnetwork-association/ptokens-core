use std::fmt;

use common::{Byte, Bytes};
use common_eth::encode_fxn_call;

use super::{UserOp, UserOpError};
use crate::SentinelError;

const GET_USER_OP_STATE_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"}],\"internalType\":\"structIStateManager.Operation\",\"name\":\"operation\",\"type\":\"tuple\"}],\"name\":\"operationStatusOf\",\"outputs\":[{\"internalType\":\"bytes1\",\"name\":\"\",\"type\":\"bytes1\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserOpSmartContractState {
    Null      = 0,
    Enqueued  = 1,
    Executed  = 2,
    Cancelled = 3,
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

impl TryFrom<Bytes> for UserOpSmartContractState {
    type Error = UserOpError;

    fn try_from(bs: Bytes) -> Result<Self, Self::Error> {
        if bs.is_empty() {
            Err(UserOpError::NotEnoughBytes)
        } else {
            Self::try_from(bs[0])
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
            user_op.encode_as_eth_abi_token()?
        ])?;
        Ok(encoded)
    }
}
