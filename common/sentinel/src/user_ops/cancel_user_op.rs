use common::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthSigningCapabilities, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, U256};

use super::{CancellationSignature, UserOp, UserOpCancellationSignature, UserOpError, UserOpUniqueId};

const CANCEL_FXN_ABI: &str = "{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"protocolFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"networkFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"forwardNetworkFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"forwardDestinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"originAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"},{\"internalType\":\"bool\",\"name\":\"isForProtocol\",\"type\":\"bool\"}],\"internalType\":\"struct IPNetworkHub.Operation\",\"name\":\"operation\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\"}],\"name\":\"protocolSentinelCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}";

impl UserOp {
    pub fn check_affordability(&self, balance: U256, gas_limit: usize, gas_price: u64) -> Result<(), UserOpError> {
        let cost = Self::get_tx_cost(gas_limit, gas_price);
        if balance > cost {
            Ok(())
        } else {
            Err(UserOpError::InsufficientBalance {
                have: balance,
                need: cost,
            })
        }
    }

    pub fn get_tx_cost(gas_limit: usize, gas_price: u64) -> U256 {
        U256::from(gas_limit as u64 * gas_price)
    }

    pub fn get_cancellation_signature(&self, pk: &EthPrivateKey) -> Result<UserOpCancellationSignature, UserOpError> {
        if self.state().is_cancelled() || self.state().is_executed() {
            Err(UserOpError::CannotCancel(Box::new(self.clone())))
        } else {
            let signer = pk.to_address();
            let uid = UserOpUniqueId::new(self.uid()?);
            let sig = CancellationSignature::from(pk.sign_hash_and_set_eth_recovery_param(self.uid()?)?);
            Ok(UserOpCancellationSignature::new(signer, uid, sig))
        }
    }

    fn encode_cancellation_fxn_data(
        &self,
        cancellation_sig: &UserOpCancellationSignature,
    ) -> Result<Bytes, UserOpError> {
        Ok(encode_fxn_call(CANCEL_FXN_ABI, "protocolSentinelCancelOperation", &[
            self.to_eth_abi_token()?,
            EthAbiToken::FixedBytes([0u8; 32].to_vec()),
            EthAbiToken::Bytes(cancellation_sig.sig().to_vec()),
        ])?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_cancellation_tx(
        &self,
        nonce: u64,
        gas_price: u64,
        gas_limit: usize,
        pnetwork_hub: &EthAddress,
        chain_id: &EthChainId,
        broadcaster_pk: &EthPrivateKey,
        cancellation_sig: &UserOpCancellationSignature,
    ) -> Result<EthTransaction, UserOpError> {
        if self.state().is_cancelled() || self.state().is_executed() {
            Err(UserOpError::CannotCancel(Box::new(self.clone())))
        } else {
            let value = 0;
            let data = self.encode_cancellation_fxn_data(cancellation_sig)?;
            Ok(
                EthTransaction::new_unsigned(data, nonce, value, *pnetwork_hub, chain_id, gas_limit, gas_price)
                    .sign(broadcaster_pk)?,
            )
        }
    }
}
