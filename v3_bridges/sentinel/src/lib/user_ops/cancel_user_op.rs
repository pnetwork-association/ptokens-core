use common::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthSignature, EthSigningCapabilities, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::Address as EthAddress;

use super::{UserOp, UserOpError};

const CANCEL_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"}],\"internalType\":\"struct IStateManager.Operation\",\"name\":\"operation\",\"type\":\"tuple\"},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\"}],\"name\":\"protocolSentinelCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

impl UserOp {
    pub fn encode_as_eth_abi_token(&self) -> Result<EthAbiToken, UserOpError> {
        Ok(EthAbiToken::Tuple(vec![
            EthAbiToken::FixedBytes(self.user_op_log.origin_block_hash()?.as_bytes().to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.origin_transaction_hash()?.as_bytes().to_vec()),
            EthAbiToken::FixedBytes(self.user_op_log.options_mask.as_bytes().to_vec()),
            EthAbiToken::Uint(self.user_op_log.nonce),
            EthAbiToken::Uint(self.user_op_log.underlying_asset_decimals),
            EthAbiToken::Uint(self.user_op_log.amount),
            EthAbiToken::Address(self.user_op_log.underlying_asset_token_address),
            EthAbiToken::FixedBytes(self.user_op_log.origin_network_id.clone().unwrap_or_default()),
            EthAbiToken::FixedBytes(self.user_op_log.destination_network_id.clone()),
            EthAbiToken::FixedBytes(self.user_op_log.underlying_asset_network_id.clone()),
            EthAbiToken::String(self.user_op_log.destination_account.clone()),
            EthAbiToken::String(self.user_op_log.underlying_asset_name.clone()),
            EthAbiToken::String(self.user_op_log.underlying_asset_symbol.clone()),
            EthAbiToken::Bytes(self.user_op_log.user_data.clone()),
        ]))
    }

    fn get_cancellation_signature(&self, pk: &EthPrivateKey) -> Result<EthSignature, UserOpError> {
        if self.state().is_cancelled() || self.state().is_executed() {
            Err(UserOpError::CannotCancel(self.state))
        } else {
            Ok(pk.sign_hash_and_set_eth_recovery_param(self.uid()?)?)
        }
    }

    fn encode_cancellation_fxn_data(&self, pk: &EthPrivateKey) -> Result<Bytes, UserOpError> {
        Ok(encode_fxn_call(CANCEL_FXN_ABI, "protocolSentinelCancelOperation", &[
            self.encode_as_eth_abi_token()?,
            EthAbiToken::Bytes(self.get_cancellation_signature(pk)?.to_vec()),
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
        core_pk: &EthPrivateKey,
        broadcaster_pk: &EthPrivateKey,
    ) -> Result<EthTransaction, UserOpError> {
        if self.state().is_cancelled() || self.state().is_executed() {
            Err(UserOpError::CannotCancel(self.state))
        } else {
            let value = 0;
            let data = self.encode_cancellation_fxn_data(core_pk)?;
            Ok(
                EthTransaction::new_unsigned(data, nonce, value, *pnetwork_hub, chain_id, gas_limit, gas_price)
                    .sign(broadcaster_pk)?,
            )
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
            .cancel(nonce, gas_price, gas_limit, &to, &pk, &chain_id)
            .unwrap();
        let expected_result = "f903af088504a817c800830186a094c2926f4e511dd26e51d5ce1231e3f26012fd1caf80b90344c0c63d580000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000032081803894d2305fd729ac0b90a4262a85c4d11b70b8bea98c40ee68bf56c8a1c2eb5cbe8387d5e9e247ea886459bcd0e599732e1a4e02a38b235cd93cac96bf300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000053900000000000000000000000089ab32156e46f46d02ade3fecbe5fc4243b9aaed01020304000000000000000000000000000000000000000000000000000000000403020100000000000000000000000000000000000000000000000000000000010303070000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000002a30784441464541343932443963363733336165336435366237456431414442363036393263393842633500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a736f6d6520746f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000353544b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c0ffee000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008401546d71a0d66df6b654d4376585f2e1ebdc0a5da76e5f9f37e00d2f19e757312fd2c09cf4a00c3c7a1a357a497ee4c252241f6fc41acb9006e500335ad0b4096610902f48be".to_string();
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
            .cancel(nonce, gas_price, gas_limit, &to, &pk, &chain_id)
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
        match user_op.cancel(nonce, gas_price, gas_limit, &to, &pk, &chain_id) {
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
        match user_op.cancel(nonce, gas_price, gas_limit, &to, &pk, &chain_id) {
            Ok(_) => panic!("should not have succeeded"),
            Err(UserOpError::CannotCancel(user_op_state)) => assert_eq!(user_op_state, user_op.state),
            Err(e) => panic!("wrong error received: {e}"),
        }
    }
}
