use common::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::Address as EthAddress;

use super::{UserOp, UserOpError};

impl UserOp {
    fn to_cancel_fxn_data(&self) -> Result<Bytes, UserOpError> {
        const CANCEL_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"}],\"internalType\":\"struct IStateManager.Operation\",\"name\":\"operation\",\"type\":\"tuple\"},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\"}],\"name\":\"protocolSentinelCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

        Ok(encode_fxn_call(CANCEL_FXN_ABI, "protocolSentinelCancelOperation", &[
            EthAbiToken::Tuple(vec![
                EthAbiToken::FixedBytes(self.block_hash.as_bytes().to_vec()),
                EthAbiToken::FixedBytes(self.tx_hash.as_bytes().to_vec()),
                EthAbiToken::FixedBytes(self.user_op_log.options_mask.as_bytes().to_vec()),
                EthAbiToken::Uint(self.user_op_log.nonce),
                EthAbiToken::Uint(self.user_op_log.underlying_asset_decimals),
                EthAbiToken::Uint(self.user_op_log.amount),
                EthAbiToken::Address(self.user_op_log.underlying_asset_token_address),
                EthAbiToken::FixedBytes(self.origin_network_id.clone()),
                EthAbiToken::FixedBytes(self.user_op_log.destination_network_id.clone()),
                EthAbiToken::FixedBytes(self.user_op_log.underlying_asset_network_id.clone()),
                EthAbiToken::String(self.user_op_log.destination_account.clone()),
                EthAbiToken::String(self.user_op_log.underlying_asset_name.clone()),
                EthAbiToken::String(self.user_op_log.underlying_asset_symbol.clone()),
                EthAbiToken::Bytes(self.user_op_log.user_data.clone()),
            ]),
            EthAbiToken::Bytes(vec![]), // NOTE: Sentinel proofs are not currently implemented
        ])?)
    }

    pub fn cancel(
        &self,
        nonce: u64,
        gas_price: u64,
        to: &EthAddress,
        gas_limit: usize,
        pk: &EthPrivateKey,
        chain_id: &EthChainId,
    ) -> Result<EthTransaction, UserOpError> {
        if self.state().is_cancelled() || self.state().is_executed() {
            Err(UserOpError::CannotCancel(self.state))
        } else {
            let value = 0;
            let data = self.to_cancel_fxn_data()?;
            Ok(EthTransaction::new_unsigned(data, nonce, value, *to, chain_id, gas_limit, gas_price).sign(pk)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use common_chain_ids::EthChainId;
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::user_ops::test_utils::{
        get_sample_cancelled_user_op,
        get_sample_enqueued_user_op,
        get_sample_executed_user_op,
        get_sample_witnessed_user_op,
    };

    fn get_sample_pk() -> EthPrivateKey {
        EthPrivateKey::try_from("64aaa58f496810ef053e25a734d1fbd90ddf5d33838bb3700014ceb59ca3204d").unwrap()
    }

    #[test]
    fn should_cancel_enqueued_user_op() {
        let nonce = 8;
        let gas_limit = 100_000;
        let pk = get_sample_pk();
        let gas_price = 20_000_000_000;
        let user_op = get_sample_enqueued_user_op();
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        let tx = user_op
            .cancel(nonce, gas_price, &to, gas_limit, &pk, &chain_id)
            .unwrap();
        let expected_result = "f903af088504a817c800830186a094c2926f4e511dd26e51d5ce1231e3f26012fd1caf80b90344c0c63d5800000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000320168fcccc67fb7419f6c4dfce71fdf3b9000f6e491e3dad94685cf62284ebdf0ac2e677e7e8c73834dc86c237f79f94ad3e4899d6aa7e561a8110a6117d13e8d50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000053900000000000000000000000089ab32156e46f46d02ade3fecbe5fc4243b9aaede15503e4000000000000000000000000000000000000000000000000000000000403020100000000000000000000000000000000000000000000000000000000010303070000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000002a30784441464541343932443963363733336165336435366237456431414442363036393263393842633500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a736f6d6520746f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000353544b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c0ffee000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008401546d72a00bc1705e0af566a83dc6e9822d089f42c16d0e00186a5ae3479501cfb88e4676a0422d64b19af138e7d2798965de638b5ac237e217bcaf8edf00d48a76e2534fbd".to_string();
        let result = tx.serialize_hex();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cancel_witnessed_user_op() {
        let nonce = 9;
        let gas_limit = 100_000;
        let pk = get_sample_pk();
        let gas_price = 20_000_000_000;
        let user_op = get_sample_witnessed_user_op();
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        let tx = user_op
            .cancel(nonce, gas_price, &to, gas_limit, &pk, &chain_id)
            .unwrap();
        let expected_result = "f9038f098504a817c800830186a094c2926f4e511dd26e51d5ce1231e3f26012fd1caf80b90324c0c63d58000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003005f4ac87dc7aad4d188fbecc0117915d99901a621d69f2a1c771f3bffc6e4b19cf6f24a42e1bfa9ab963786a9d2e146da7a6afad0ed188daa7a88e37bf42db789000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000195e700000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000056bc75e2d6310000000000000000000000000000025a7fc8a2400d9aaafe149750c176a4d84a666c0e15503e400000000000000000000000000000000000000000000000000000000953835d900000000000000000000000000000000000000000000000000000000953835d90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000002a307861343136353762663232354638456337453230313043383963334630383431373239343832363444000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005546f6b656e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003544b4e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008401546d71a0bdb919be60086a45bc95cdc5bff80b4efdbb63994876990f0fbfced290d9e618a0432eeeb36bc652ae5d8f21e0e923a1f73ffb0014a80e0b1c23df565375a8452c".to_string();
        let result = tx.serialize_hex();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_able_to_cancel_cancelled_user_op() {
        let user_op = get_sample_cancelled_user_op();
        assert!(user_op.state().is_cancelled());
        let nonce = 9;
        let gas_limit = 100_000;
        let pk = get_sample_pk();
        let gas_price = 20_000_000_000;
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        match user_op.cancel(nonce, gas_price, &to, gas_limit, &pk, &chain_id) {
            Ok(_) => panic!("should not have succeeded"),
            Err(UserOpError::CannotCancel(user_op_state)) => assert_eq!(user_op_state, user_op.state),
            Err(e) => panic!("wrong error received: {e}"),
        }
    }

    #[test]
    fn should_not_be_able_to_cancel_executed_user_op() {
        let user_op = get_sample_executed_user_op();
        assert!(user_op.state().is_executed());
        let nonce = 9;
        let gas_limit = 100_000;
        let pk = get_sample_pk();
        let gas_price = 20_000_000_000;
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        match user_op.cancel(nonce, gas_price, &to, gas_limit, &pk, &chain_id) {
            Ok(_) => panic!("should not have succeeded"),
            Err(UserOpError::CannotCancel(user_op_state)) => assert_eq!(user_op_state, user_op.state),
            Err(e) => panic!("wrong error received: {e}"),
        }
    }
}
