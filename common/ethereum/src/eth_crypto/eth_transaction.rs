use common::types::{Byte, Bytes, Result};
use common_chain_ids::EthChainId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, U256};
use rlp::RlpStream;
use serde::{Deserialize, Serialize};

use crate::{
    encode_erc777_mint_fxn_maybe_with_data,
    EthPrivateKey,
    EthSignature,
    EthSigningCapabilities,
    EthTxInfoCompatible,
    RelayTransaction,
    VALUE_FOR_MINTING_TX,
};

#[derive(Debug, Clone, Eq, PartialEq, Default, Deref, Constructor, Serialize, Deserialize)]
pub struct EthTransactions(pub Vec<EthTransaction>);

impl EthTransactions {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct EthTransaction {
    pub v: u64,
    pub r: U256,
    pub s: U256,
    pub to: Bytes,
    pub nonce: U256,
    pub value: U256,
    pub data: Bytes,
    pub gas_limit: U256,
    pub gas_price: U256,
    pub chain_id: EthChainId,
}

impl EthTransaction {
    pub fn from_bytes(tx_bytes: &[Byte]) -> Result<EthTransaction> {
        let decoded_tx: Vec<Bytes> = rlp::decode_list(tx_bytes);
        if decoded_tx.len() != 9 {
            // FIXME Magic number (Well, it's the number of elements in a signed ETH tx...)!
            Err("Error decoded ETH tx!".into())
        } else {
            Ok(EthTransaction {
                nonce: U256::from_big_endian(&decoded_tx[0]),
                gas_price: U256::from_big_endian(&decoded_tx[1]),
                gas_limit: U256::from_big_endian(&decoded_tx[2]),
                to: decoded_tx[3].clone(),
                value: U256::from_big_endian(&decoded_tx[4]),
                data: decoded_tx[5].clone(),
                v: 0, // NOTE: Not calculated
                r: U256::from_big_endian(&decoded_tx[7]),
                s: U256::from_big_endian(&decoded_tx[8]),
                chain_id: EthChainId::default(), // NOTE: This isn't calculated!
            })
        }
    }
}

impl EthTransaction {
    pub fn new_unsigned(
        data: Bytes,
        nonce: u64,
        value: usize,
        to: EthAddress,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
    ) -> EthTransaction {
        Self::new_eth_tx(
            to.as_bytes().to_vec(),
            data,
            nonce,
            value,
            chain_id,
            gas_limit,
            gas_price,
        )
    }

    pub fn new_eth_tx(
        to: Bytes,
        data: Bytes,
        nonce: u64,
        value: usize,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
    ) -> EthTransaction {
        EthTransaction {
            to,
            data,
            r: U256::zero(),
            s: U256::zero(),
            nonce: nonce.into(),
            value: value.into(),
            v: chain_id.to_u64(), // Per EIP155
            chain_id: chain_id.clone(),
            gas_limit: gas_limit.into(),
            gas_price: gas_price.into(),
        }
    }

    fn add_signature_to_transaction(mut self, sig: EthSignature) -> Self {
        self.r = sig[0..32].into();
        self.s = sig[32..64].into();
        self.v = Self::calculate_v_from_chain_id(sig[64], &self.chain_id);
        self
    }

    fn calculate_v_from_chain_id(sig_v: u8, chain_id: &EthChainId) -> u64 {
        chain_id.to_u64() * 2 + sig_v as u64 + 35 // Per EIP155
    }

    pub fn sign<T: EthSigningCapabilities>(self, pk: &T) -> Result<Self> {
        pk.hash_and_sign_msg(&self.serialize_bytes())
            .map(|sig| self.add_signature_to_transaction(sig))
    }

    pub fn serialize_hex(&self) -> String {
        hex::encode(self.serialize_bytes())
    }

    pub fn to_bytes(&self) -> Bytes {
        self.serialize_bytes()
    }
}

impl EthTxInfoCompatible for EthTransaction {
    fn is_any_sender(&self) -> bool {
        false
    }

    fn any_sender_tx(&self) -> Option<RelayTransaction> {
        None
    }

    fn eth_tx_hex(&self) -> Option<String> {
        Some(self.serialize_hex())
    }

    fn serialize_bytes(&self) -> Bytes {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.begin_list(9);
        rlp_stream.append(&self.nonce);
        rlp_stream.append(&self.gas_price);
        rlp_stream.append(&self.gas_limit);
        rlp_stream.append(&self.to);
        rlp_stream.append(&self.value);
        rlp_stream.append(&self.data);
        rlp_stream.append(&self.v);
        rlp_stream.append(&self.r);
        rlp_stream.append(&self.s);
        rlp_stream.out().to_vec()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_unsigned_minting_tx(
    nonce: u64,
    amount: &U256,
    chain_id: &EthChainId,
    to: EthAddress,
    gas_price: u64,
    recipient: &EthAddress,
    user_data: Option<Bytes>,
    operator_data: Option<Bytes>,
) -> Result<EthTransaction> {
    let gas_limit = if user_data.is_some() {
        chain_id.get_erc777_mint_with_data_gas_limit()
    } else {
        chain_id.get_erc777_mint_with_no_data_gas_limit()
    };
    Ok(EthTransaction::new_unsigned(
        encode_erc777_mint_fxn_maybe_with_data(recipient, amount, user_data, operator_data)?,
        nonce,
        VALUE_FOR_MINTING_TX,
        to,
        chain_id,
        gas_limit,
        gas_price,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn get_signed_minting_tx(
    amount: &U256,
    nonce: u64,
    chain_id: &EthChainId,
    to: EthAddress,
    gas_price: u64,
    recipient: &EthAddress,
    eth_private_key: &EthPrivateKey,
    user_data: Option<Bytes>,
    operator_data: Option<Bytes>,
) -> Result<EthTransaction> {
    get_unsigned_minting_tx(
        nonce,
        amount,
        chain_id,
        to,
        gas_price,
        recipient,
        user_data,
        operator_data,
    )?
    .sign(eth_private_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_sample_eth_address, get_sample_eth_private_key, get_sample_unsigned_eth_transaction};

    #[test]
    fn should_serialize_simple_eth_tx_to_bytes() {
        let expected_result = vec![
            229, 128, 133, 4, 168, 23, 200, 0, 131, 1, 134, 160, 148, 83, 194, 4, 141, 173, 79, 207, 171, 68, 195, 239,
            61, 22, 232, 130, 181, 23, 141, 244, 43, 1, 128, 4, 128, 128,
        ];
        let tx = get_sample_unsigned_eth_transaction();
        let result = tx.serialize_bytes();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_sign_simple_eth_tx() {
        // NOTE: Real tx was broadcast here: https://rinkeby.etherscan.io/tx/0xd293dc1bad03b7c3c76845474dd9e47b6a2d218590030926a3841030f07ff3db
        let expected_result = "f865808504a817c800830186a09453c2048dad4fcfab44c3ef3d16e882b5178df42b01802ca08f29776b90079ba489419a7e2db5910a472056cf7d5fdf9bc3fc4b919d3feefea03351a3ec56d36d88b4714e78a7045c74acaeb1a66ffe5d27b229a0a5a13d4d91"
            .to_string();
        let private_key = get_sample_eth_private_key();
        let tx = get_sample_unsigned_eth_transaction();
        let result = tx.sign(&private_key).unwrap().serialize_hex();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serde_signed_txs() {
        let private_key = get_sample_eth_private_key();
        let tx = get_sample_unsigned_eth_transaction();
        let signed_tx = tx.sign(&private_key).unwrap();
        let signed_txs = EthTransactions::new(vec![signed_tx]);
        let bytes = signed_txs.to_bytes().unwrap();
        let result = EthTransactions::from_bytes(&bytes).unwrap();
        assert_eq!(result, signed_txs);
    }

    #[test]
    fn should_get_unsigned_minting_tx() {
        let recipient = get_sample_eth_address();
        let amount = U256::from_dec_str("1").unwrap();
        let nonce = 4;
        let chain_id = EthChainId::Rinkeby;
        let gas_price = 20_000_000_000;
        let test_contract_address = "c63b099efB18c8db573981fB64564f1564af4f30";
        let to = EthAddress::from_slice(&hex::decode(test_contract_address).unwrap());
        let user_data = None;
        let operator_data = None;
        let result = get_unsigned_minting_tx(
            nonce,
            &amount,
            &chain_id,
            to,
            gas_price,
            &recipient,
            user_data,
            operator_data,
        )
        .unwrap();
        let expected_result = "f86a048504a817c8008302bf2094c63b099efb18c8db573981fb64564f1564af4f3080b84440c10f190000000000000000000000001739624f5cd969885a224da84418d12b8570d61a0000000000000000000000000000000000000000000000000000000000000001048080"
            .to_string();
        assert_eq!(result.serialize_hex(), expected_result);
    }

    #[test]
    fn should_get_signed_minting_tx() {
        let recipient = get_sample_eth_address();
        let amount = U256::from_dec_str("1").unwrap();
        let nonce = 5;
        let chain_id = EthChainId::Rinkeby;
        let gas_price = 20_000_000_000;
        let eth_private_key = get_sample_eth_private_key();
        let test_contract_address = "c63b099efB18c8db573981fB64564f1564af4f30";
        let to = EthAddress::from_slice(&hex::decode(test_contract_address).unwrap());
        let user_data = None;
        let operator_data = None;
        let result = get_signed_minting_tx(
            &amount,
            nonce,
            &chain_id,
            to,
            gas_price,
            &recipient,
            &eth_private_key,
            user_data,
            operator_data,
        )
        .unwrap();
        let expected_result = "f8aa058504a817c8008302bf2094c63b099efb18c8db573981fb64564f1564af4f3080b84440c10f190000000000000000000000001739624f5cd969885a224da84418d12b8570d61a00000000000000000000000000000000000000000000000000000000000000012ca08be9eb0f9ce398001121ae610b67be2fe55ef291a492100c968b32ace0100b78a01e9b042d110be12e5c4337f5e531f45a7ca51860af3aa7a4c95fa6b44bba332a"
            .to_string();
        let expected_tx_hash = "6b56132c3b31d5e87af53547e5e0edaef34eb7ae45e850b08d520c07a589b14b";
        let tx_hash = result.get_tx_hash();
        assert_eq!(tx_hash, expected_tx_hash);
        assert_eq!(result.serialize_hex(), expected_result);
    }
}
