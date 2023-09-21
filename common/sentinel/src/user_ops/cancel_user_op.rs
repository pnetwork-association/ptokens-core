use common::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{encode_fxn_call, EthPrivateKey, EthSigningCapabilities, EthTransaction};
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, U256};

use super::{CancellationSignature, UserOp, UserOpCancellationSignature, UserOpError, UserOpUniqueId};

const CANCEL_FXN_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"originBlockHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"originTransactionHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"optionsMask\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"underlyingAssetDecimals\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"assetAmount\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"underlyingAssetTokenAddress\",\"type\":\"address\"},{\"internalType\":\"bytes4\",\"name\":\"originNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"destinationNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"bytes4\",\"name\":\"underlyingAssetNetworkId\",\"type\":\"bytes4\"},{\"internalType\":\"string\",\"name\":\"destinationAccount\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetName\",\"type\":\"string\"},{\"internalType\":\"string\",\"name\":\"underlyingAssetSymbol\",\"type\":\"string\"},{\"internalType\":\"bytes\",\"name\":\"userData\",\"type\":\"bytes\"}],\"internalType\":\"struct IStateManager.Operation\",\"name\":\"operation\",\"type\":\"tuple\"},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\"}],\"name\":\"protocolSentinelCancelOperation\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

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
        todo!("FIXME update abi & impl of this");
        // FIXME abi needs updating!
        Ok(encode_fxn_call(CANCEL_FXN_ABI, "protocolSentinelCancelOperation", &[
            self.encode_as_eth_abi_token()?,
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

    fn get_sample_pk_1() -> EthPrivateKey {
        EthPrivateKey::try_from("64aaa58f496810ef053e25a734d1fbd90ddf5d33838bb3700014ceb59ca3204d").unwrap()
    }
    fn get_sample_pk_2() -> EthPrivateKey {
        EthPrivateKey::try_from("73fa7fa1bed876c0694483f316180e4fca9f0d1d0323e19ee0d53adecdec94b9").unwrap()
    }

    #[test]
    fn should_cancel_enqueued_user_op() {
        let nonce = 8;
        let gas_limit = 100_000;
        let core_pk = get_sample_pk_1();
        let broadcaster_pk = get_sample_pk_2();
        let gas_price = 20_000_000_000;
        let user_op = get_sample_enqueued_user_op();
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        let tx = user_op
            .get_cancellation_tx(nonce, gas_price, gas_limit, &to, &chain_id, &core_pk, &broadcaster_pk)
            .unwrap();
        let expected_result = "f9040f088504a817c800830186a094c2926f4e511dd26e51d5ce1231e3f26012fd1caf80b903a4c0c63d580000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000032081803894d2305fd729ac0b90a4262a85c4d11b70b8bea98c40ee68bf56c8a1c2eb5cbe8387d5e9e247ea886459bcd0e599732e1a4e02a38b235cd93cac96bf300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000053900000000000000000000000089ab32156e46f46d02ade3fecbe5fc4243b9aaed01020304000000000000000000000000000000000000000000000000000000000403020100000000000000000000000000000000000000000000000000000000010303070000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000002a30784441464541343932443963363733336165336435366237456431414442363036393263393842633500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a736f6d6520746f6b656e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000353544b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c0ffee000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000410524ab9c4107a052047b079aa7dadc2a065d437ccc3e8bf2059c17b47c0c663112768715614ebb46fa10d19089bb5008376056a6a0e5b9c023767147741273e61b000000000000000000000000000000000000000000000000000000000000008401546d72a081d1fb432c95da7ada423b4f431b2ed140cce10df101074a98464e67a3a776c6a050d8824037fd0626c5ee96987cb5ea5a9b7ce37073bab172befcab320aeef580".to_string();
        let result = tx.serialize_hex();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cancel_witnessed_user_op() {
        let nonce = 9;
        let gas_limit = 100_000;
        let core_pk = get_sample_pk_1();
        let broadcaster_pk = get_sample_pk_2();
        let gas_price = 20_000_000_000;
        let user_op = get_sample_witnessed_user_op();
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        let tx = user_op
            .get_cancellation_tx(nonce, gas_price, gas_limit, &to, &chain_id, &core_pk, &broadcaster_pk)
            .unwrap();
        let expected_result = "f903ef098504a817c800830186a094c2926f4e511dd26e51d5ce1231e3f26012fd1caf80b90384c0c63d58000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003005f4ac87dc7aad4d188fbecc0117915d99901a621d69f2a1c771f3bffc6e4b19cf6f24a42e1bfa9ab963786a9d2e146da7a6afad0ed188daa7a88e37bf42db789000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000195e700000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000056bc75e2d6310000000000000000000000000000025a7fc8a2400d9aaafe149750c176a4d84a666c0e15503e400000000000000000000000000000000000000000000000000000000953835d900000000000000000000000000000000000000000000000000000000953835d90000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000002a307861343136353762663232354638456337453230313043383963334630383431373239343832363444000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005546f6b656e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003544b4e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000041b39ad64bb5f23647cac922a204d5050c84d913faa497926b72ec054d0a1d5a942d7a6e43d2d84b21ce15d4bae0e024cba924659908273b3ec8411728ada5f9a91b000000000000000000000000000000000000000000000000000000000000008401546d72a04cc18f405e849c3c7b0190b7bebec9c1d751b796268f3a01882f7897fcd454a4a0044ed164d9ca26ca7293771e8e7e8271f7e8d3d53273c496cf1c080d2a5c6de5".to_string();
        let result = tx.serialize_hex();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_able_to_cancel_cancelled_user_op() {
        let user_op = get_sample_cancelled_user_op();
        assert!(user_op.state().is_cancelled());
        let nonce = 9;
        let gas_limit = 100_000;
        let core_pk = get_sample_pk_1();
        let broadcaster_pk = get_sample_pk_2();
        let gas_price = 20_000_000_000;
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        match user_op.get_cancellation_tx(nonce, gas_price, gas_limit, &to, &chain_id, &core_pk, &broadcaster_pk) {
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
        let core_pk = get_sample_pk_1();
        let broadcaster_pk = get_sample_pk_2();
        let gas_price = 20_000_000_000;
        let to = convert_hex_to_eth_address("0xc2926f4e511dd26e51d5ce1231e3f26012fd1caf").unwrap();
        let chain_id = EthChainId::Sepolia;
        match user_op.get_cancellation_tx(nonce, gas_price, gas_limit, &to, &chain_id, &core_pk, &broadcaster_pk) {
            Ok(_) => panic!("should not have succeeded"),
            Err(UserOpError::CannotCancel(user_op_state)) => assert_eq!(user_op_state, user_op.state),
            Err(e) => panic!("wrong error received: {e}"),
        }
    }
}
