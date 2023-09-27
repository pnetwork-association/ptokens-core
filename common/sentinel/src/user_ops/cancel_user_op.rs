use common::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthSigningCapabilities, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, U256};

use super::{CancellationSignature, UserOp, UserOpCancellationSignature, UserOpError, UserOpUniqueId};

const CANCEL_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"protocolFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"networkFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"forwardNetworkFeeAssetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"forwardDestinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"originAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"},{\"internalType\":\"bool\",\"name\":\"isForProtocol\",\"type\":\"bool\"}],\"internalType\":\"struct IPNetworkHub.Operation\",\"name\":\"operation\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bytes\",\"name\":\"signature\",\"type\":\"bytes\"}],\"name\":\"protocolSentinelCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

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
        let user_op_tuple_token = self.to_eth_abi_token()?;
        // FIXME Need to generate actual proof
        let proof = EthAbiToken::Array(vec![EthAbiToken::FixedBytes([0u8; 32].to_vec())]);
        let sig = EthAbiToken::Bytes(cancellation_sig.sig().to_vec());
        debug!("cancellation tx sig: {sig:?}");
        debug!("cancellation tx proof: {proof:?}");

        let r = encode_fxn_call(CANCEL_FXN_ABI, "protocolSentinelCancelOperation", &[
            user_op_tuple_token,
            proof,
            sig,
        ])?;

        Ok(r)
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
            debug!("nonce: {nonce}");
            debug!("gas_price: {gas_price}");
            debug!("gas_limit: {gas_limit}");
            debug!("pnetwork_hub: {pnetwork_hub}");
            debug!("chain_id: {chain_id}");
            debug!("tx signer: {}", broadcaster_pk.to_address());
            debug!("cancellation sig: {}", cancellation_sig.clone().to_string());
            Ok(
                EthTransaction::new_unsigned(data, nonce, value, *pnetwork_hub, chain_id, gas_limit, gas_price)
                    .sign(broadcaster_pk)?,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::BridgeSide;
    use common_eth::convert_hex_to_eth_address;
    use common_metadata::MetadataChainId;

    use super::*;
    use crate::{
        user_ops::{test_utils::get_sub_mat_with_enqueued_user_op, UserOps},
        NetworkId,
    };

    #[test]
    fn should_get_user_op_cancellation_tx_correctly() {
        let side = BridgeSide::Host;
        let eth_chain_id = EthChainId::PolygonMainnet;
        let origin_network_id = NetworkId::try_from(MetadataChainId::BscMainnet).unwrap();
        let pnetwork_hub = convert_hex_to_eth_address("0xD2BAC275ffFdbDD23Ecea72f4B161b3aF90300A3").unwrap();
        let sub_mat = get_sub_mat_with_enqueued_user_op();
        let ops = UserOps::from_sub_mat(side, &origin_network_id.to_bytes(), &pnetwork_hub, &sub_mat).unwrap();
        assert_eq!(ops.len(), 1);
        let op = ops[0].clone();
        let nonce = 0;
        let gas_price = 1;
        let gas_limit = 2;
        let pk = EthPrivateKey::from_str("adcf1671004483793d26c395fea5e3883339f7abc2b053ee6c995b7315708f2d").unwrap();
        let cancellation_sig = op.get_cancellation_signature(&pk).unwrap();
        let tx = op
            .get_cancellation_tx(
                nonce,
                gas_price,
                gas_limit,
                &pnetwork_hub,
                &eth_chain_id,
                &pk,
                &cancellation_sig,
            )
            .unwrap();
        let hex = tx.serialize_hex();
        let expected_hex = "f9056580010294d2bac275fffdbdd23ecea72f4b161b3af90300a380b90504732076ce00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000440000000000000000000000000000000000000000000000000000000000000048005cf0e83408207704ee0ea2a4a6ea87905fc0d2038dbb610a0ca64f2cf47b134b1bb8b6502edc17fdd0cc83505289a6d429a6381ffe5dbf4fe31a88dd236d64300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018012000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000a2992000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000007d0000000000000000000000000daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af925aca268b00000000000000000000000000000000000000000000000000000000f9b459a100000000000000000000000000000000000000000000000000000000b9286154000000000000000000000000000000000000000000000000000000005aca268b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000340000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786464623566343533353132336461613561653334336332343030366634303735616261663566376200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786444623566343533353132334441613561453334336332343030364634303735614241463546374200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e704e6574776f726b20546f6b656e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003504e54000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000413db751a13d564e04ff41538b341ee7b8480c3af1aaa686e6f2761f05882edb703604191d5a4e53436dc10caa6271c8064c085ee47f84ee57aa94465f2f5d8d381b00000000000000000000000000000000000000000000000000000000000000820135a0cfa3d667f989117e00778d94ed81700a18dc764451382a9764e4fe3df952114ba07fb2eacf12125c04d56e8f3605bcd721455da7552fbe415b505815db62da9777";
        assert_eq!(hex, expected_hex);
    }
}
