#[cfg(test)]
use std::str::FromStr;

use common::types::{Byte, Bytes, Result};
#[cfg(test)]
use common::{strip_hex_prefix, AppError};
use common_chain_ids::EthChainId;
use derive_more::Deref;
use ethabi::{encode, Token};
use ethereum_types::{Address as EthAddress, Signature as EthSignature, U256};
use rlp::RlpStream;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::convert_hex_to_eth_address;
use crate::{
    any_sender::{
        relay_contract::RelayContract,
        serde::{compensation, data},
    },
    eth_contracts::encode_mint_by_proxy_tx_data,
    eth_crypto::EthPrivateKey,
    eth_traits::{EthSigningCapabilities, EthTxInfoCompatible},
};

pub const ANY_SENDER_GAS_LIMIT: u32 = 300_000;
pub const ANY_SENDER_MAX_DATA_LEN: usize = 3_000;
pub const ANY_SENDER_MAX_GAS_LIMIT: u32 = 3_000_000;
pub const ANY_SENDER_DEFAULT_DEADLINE: Option<u64> = None;
pub const ANY_SENDER_MAX_COMPENSATION_WEI: u64 = 49_999_999_999_999_999;

#[derive(Clone, Debug, Eq, PartialEq, Deref, Serialize, Deserialize)]
pub struct RelayTransactions(pub Vec<RelayTransaction>);

/// The standard eth chain id.
/// An AnySender relay transaction. It is very similar
/// to a normal transaction except for a few fields.
/// The schema can be found [here](https://github.com/PISAresearch/docs.any.sender/blob/master/docs/relayTx.schema.json).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayTransaction {
    /// The standard eth chain id.
    /// Currently supports Ropsten = 3 and Mainnet = 1.
    chain_id: EthChainId,

    /// The ethereum address of the user
    /// authorising this relay transaction.
    pub from: EthAddress,

    /// A signature made by the `from` authority
    /// over the full relay transaction data.
    /// Using this [digest](https://github.com/PISAresearch/contracts.any.sender/blob/e7d9cf8c26bdcae67e39f464b4a102a8572ff468/versions/0.2.1/contracts/core/RelayTxStruct.sol#L22).
    pub signature: EthSignature,

    /// The ABI encoded call data.
    /// Same as standard Ethereum.
    /// Max data length is 3000 bytes (BETA).
    #[serde(with = "data")]
    pub data: Bytes,

    /// The block by which this transaction must be mined.
    /// Must be at most 400 blocks larger than the current block height (BETA).
    /// There is a tolerance of 20 blocks above and below this value (BETA).
    /// Can optionally be set to 0. In this case the AnySender API will
    /// fill in a deadline (currentBlock + 400) and populate it in the returned receipt.
    // An integer in range 0..=(currentBlock + 400).
    pub deadline: u64,

    /// The gas limit provided to the transaction for execution.
    /// Same as standard Ethereum.
    /// An integer in range 0..=3.000.000 (BETA).
    pub gas_limit: u32,

    /// The value of the compensation that the user will be owed
    /// if AnySender fails to mine the transaction
    /// before the `deadline`.
    /// Max compensation is 0.05 ETH (BETA).
    // Maximum value 50_000_000_000_000_000
    #[serde(with = "compensation")]
    pub compensation: u64,

    /// The address of the relay contract
    /// that will be used to relay this transaction.
    pub relay_contract_address: EthAddress,

    /// The address the transaction is directed to.
    /// Cannot be empty.
    pub to: EthAddress,
}

impl RelayTransactions {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[cfg(test)]
impl FromStr for RelayTransaction {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        RelayTransactionJson::from_str(s).and_then(Self::from_json)
    }
}

impl RelayTransaction {
    #[cfg(test)]
    pub fn from_json(json: RelayTransactionJson) -> Result<Self> {
        json.to_relay_transaction()
    }

    #[cfg(test)]
    pub fn to_string(&self) -> Result<String> {
        RelayTransactionJson::from_relay_transaction(self).and_then(|json| json.to_string())
    }

    /// Creates a new signed relay transaction.
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        from: EthAddress,
        chain_id: &EthChainId,
        eth_private_key: EthPrivateKey,
        data: Bytes,
        deadline: Option<u64>,
        gas_limit: u32,
        compensation: u64,
        to: EthAddress,
    ) -> Result<RelayTransaction> {
        let relay_contract_address = RelayContract::from_eth_chain_id(chain_id)?.address()?;
        let relay_transaction = RelayTransaction::new_unsigned(
            chain_id,
            from,
            data,
            deadline,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        )?
        .sign(&eth_private_key)?;

        info!("✔ AnySender transaction signature is calculated. Returning signed transaction...");

        Ok(relay_transaction)
    }

    /// Creates a new unsigned relay transaction from data.
    #[allow(clippy::too_many_arguments)]
    fn new_unsigned(
        chain_id: &EthChainId,
        from: EthAddress,
        data: Bytes,
        deadline: Option<u64>,
        gas_limit: u32,
        compensation: u64,
        relay_contract_address: EthAddress,
        to: EthAddress,
    ) -> Result<RelayTransaction> {
        info!("✔ Checking AnySender transaction constraints...");

        let deadline = deadline.unwrap_or_default();

        if gas_limit > ANY_SENDER_MAX_GAS_LIMIT {
            return Err("✘ AnySender gas limit is out of range!".into());
        }

        if data.len() > ANY_SENDER_MAX_DATA_LEN {
            return Err("✘ AnySender data length is out of range!".into());
        }

        if compensation > ANY_SENDER_MAX_COMPENSATION_WEI {
            return Err("✘ AnySender compensation should be smaller than 0.05 ETH!".into());
        }

        if *chain_id != EthChainId::Mainnet && *chain_id != EthChainId::Ropsten {
            return Err("✘ AnySender is not available on chain with the id provided!".into());
        }

        info!("✔ AnySender transaction constraints are satisfied. Returning unsigned transaction...");

        Ok(RelayTransaction {
            to,
            from,
            data,
            deadline,
            gas_limit,
            compensation,
            relay_contract_address,
            chain_id: chain_id.clone(),
            signature: EthSignature::default(),
        })
    }

    /// Calculates AnySender relay transaction signature.
    fn sign(mut self, eth_private_key: &EthPrivateKey) -> Result<RelayTransaction> {
        info!("Calculating relay transaction signature...");

        let transaction_bytes = encode(&[
            Token::Address(self.to),
            Token::Address(self.from),
            Token::Bytes(self.data.clone()),
            Token::Uint(self.deadline.into()),
            Token::Uint(self.compensation.into()),
            Token::Uint(self.gas_limit.into()),
            Token::Uint(U256::from(self.chain_id.to_u64())),
            Token::Address(self.relay_contract_address),
        ]);

        // TODO: Check that this should be using a prefixed msg signature.
        let signed_message = eth_private_key.hash_and_sign_msg_with_eth_prefix(&transaction_bytes)?;
        self.signature = EthSignature::from_slice(&signed_message.to_vec());

        Ok(self)
    }

    /// Creates a new AnySender relayed `mintByProxy` ERC777 proxy contract transaction.
    pub fn new_mint_by_proxy_tx(
        chain_id: &EthChainId,
        from: EthAddress,
        token_amount: U256,
        any_sender_nonce: u64,
        eth_private_key: &EthPrivateKey,
        to: EthAddress,
        token_recipient: EthAddress,
    ) -> Result<RelayTransaction> {
        RelayTransaction::new_unsigned(
            chain_id,
            from,
            encode_mint_by_proxy_tx_data(eth_private_key, token_recipient, token_amount, any_sender_nonce)?,
            ANY_SENDER_DEFAULT_DEADLINE,
            ANY_SENDER_GAS_LIMIT,
            ANY_SENDER_MAX_COMPENSATION_WEI,
            RelayContract::from_eth_chain_id(chain_id)?.address()?,
            to,
        )?
        .sign(eth_private_key)
    }

    #[cfg(test)]
    pub fn serialize_hex(&self) -> String {
        hex::encode(self.serialize_bytes())
    }
}

impl EthTxInfoCompatible for RelayTransaction {
    fn is_any_sender(&self) -> bool {
        true
    }

    fn any_sender_tx(&self) -> Option<RelayTransaction> {
        Some(self.clone())
    }

    fn eth_tx_hex(&self) -> Option<String> {
        None
    }

    fn serialize_bytes(&self) -> Bytes {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.begin_list(9);
        rlp_stream.append(&self.to);
        rlp_stream.append(&self.from);
        rlp_stream.append(&self.data);
        rlp_stream.append(&self.deadline);
        rlp_stream.append(&self.compensation);
        rlp_stream.append(&self.gas_limit);
        rlp_stream.append(&self.chain_id.to_bytes().unwrap()); // FIXME
        rlp_stream.append(&self.relay_contract_address);
        rlp_stream.append(&self.signature);
        rlp_stream.out().to_vec()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayTransactionJson {
    chain_id: u64,
    from: String,
    signature: String,
    data: String,
    deadline: u64,
    gas_limit: u32,
    compensation: String,
    relay_contract_address: String,
    to: String,
}

#[cfg(test)]
impl RelayTransactionJson {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn to_relay_transaction(&self) -> Result<RelayTransaction> {
        Ok(RelayTransaction {
            deadline: self.deadline,
            gas_limit: self.gas_limit,
            to: convert_hex_to_eth_address(&self.to)?,
            from: convert_hex_to_eth_address(&self.from)?,
            chain_id: EthChainId::try_from(self.chain_id)?,
            compensation: self.compensation.parse::<u64>()?,
            data: hex::decode(strip_hex_prefix(&self.data))?,
            relay_contract_address: convert_hex_to_eth_address(&self.relay_contract_address)?,
            signature: EthSignature::from_slice(&hex::decode(strip_hex_prefix(&self.signature))?),
        })
    }

    pub fn from_relay_transaction(relay_transaction: &RelayTransaction) -> Result<Self> {
        Ok(Self {
            deadline: relay_transaction.deadline,
            gas_limit: relay_transaction.gas_limit,
            chain_id: relay_transaction.chain_id.to_u64(),
            to: format!("0x{}", hex::encode(relay_transaction.to)),
            data: format!("0x{}", hex::encode(&relay_transaction.data)),
            from: format!("0x{}", hex::encode(relay_transaction.from)),
            compensation: format!("{}", relay_transaction.compensation),
            signature: format!("0x{}", hex::encode(relay_transaction.signature)),
            relay_contract_address: format!("0x{}", hex::encode(relay_transaction.relay_contract_address)),
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_unsigned_eth_transaction;

    #[test]
    fn should_create_new_signed_relay_tx_from_data() {
        let chain_id = EthChainId::Ropsten;
        let data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000").unwrap();
        let deadline = Some(0);
        let gas_limit = 100000;
        let compensation = 500000000;
        let relay_contract_address = RelayContract::Ropsten.address().unwrap();
        let to = EthAddress::from_slice(&hex::decode("FDE83bd51bddAA39F15c1Bf50E222a7AE5831D83").unwrap());

        let expected_data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000").unwrap();

        // private key without recovery param
        let eth_private_key =
            EthPrivateKey::from_str("841734cb439af03575c37c29b332619f3da9ea2fbaed58a1c8b1188ecff2a8dd").unwrap();
        let from = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());

        let relay_transaction = RelayTransaction::new(
            from,
            &chain_id,
            eth_private_key,
            data.clone(),
            deadline,
            gas_limit,
            compensation,
            to,
        )
        .unwrap();

        let expected_signature = EthSignature::from_slice(
            &hex::decode("983fa37fae8243405e58dfa71a7f1aa01ce0d7f0658708d201825fb6b2e0280625b37f5000c3f74a96011ac50450b750d75ec7a5c634b815e2645af93bd2a4041c")
                .unwrap(),
        );
        let expected_relay_transaction = RelayTransaction {
            signature: expected_signature,
            data: expected_data.clone(),
            chain_id: chain_id.clone(),
            deadline: 0,
            from,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        };

        assert_eq!(relay_transaction, expected_relay_transaction);

        // private key with recovery param
        let eth_private_key =
            EthPrivateKey::from_str("0637a2ddfec66c14670c5d7be2e847468bd429364184129eca0e89e2ae3f0b2d").unwrap();

        let from = EthAddress::from_slice(&hex::decode("1a96829d85bdf719b58b2593e2853d4ae5a0f50b").unwrap());

        let relay_transaction = RelayTransaction::new(
            from,
            &chain_id,
            eth_private_key,
            data,
            deadline,
            gas_limit,
            compensation,
            to,
        )
        .unwrap();

        let expected_signature = EthSignature::from_slice(
            &hex::decode("91875195bc6a836cd136c75b9d4fcc466c053e6cab9deee4daa17bdf4020e2ea75dcfedd97786476ad624ac7c8c8ea8dd512eeb8c75bc7e20c820dbc12bd44121b")
                .unwrap()
        );
        let expected_relay_transaction = RelayTransaction {
            signature: expected_signature,
            data: expected_data,
            chain_id,
            deadline: 0,
            from,
            gas_limit,
            compensation,
            relay_contract_address,
            to,
        };
        assert_eq!(relay_transaction, expected_relay_transaction);
    }

    #[test]
    fn should_create_new_any_sender_relayed_mint_by_proxy_tx() {
        let eth_transaction = get_sample_unsigned_eth_transaction();
        let chain_id = EthChainId::Ropsten;
        let eth_private_key =
            EthPrivateKey::from_str("841734cb439af03575c37c29b332619f3da9ea2fbaed58a1c8b1188ecff2a8dd").unwrap();
        let from = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let any_sender_nonce = 0;
        let amount = U256::from(1337);

        let relay_transaction = RelayTransaction::new_mint_by_proxy_tx(
            &chain_id,
            from,
            amount,
            any_sender_nonce,
            &eth_private_key,
            EthAddress::from_slice(&eth_transaction.to),
            EthAddress::from_slice(&eth_transaction.to), // FIXME This should be a different address really!
        )
        .expect("Error creating AnySender relay transaction from eth transaction!");
        let data = hex::decode("7ad6ae4700000000000000000000000053c2048dad4fcfab44c3ef3d16e882b5178df42b00000000000000000000000000000000000000000000000000000000000005390000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000416c7739aefe46a4bbef64ea98ff3719204b2e23b0b45f7b213642b1ec13b3021f47a5b6c3f5f1b8dd60c37014eb1403f85bf2c586529927674800609fe5582d261c00000000000000000000000000000000000000000000000000000000000000").unwrap();
        let signature = EthSignature::from_slice(
            &hex::decode("51c51f757d792a8cebcfdaf00fd8e4c8cd231b44290300b040b9be6932bc9f0251f2e965a6526b52476ca3cbaa04f7b7564a018561584fb80d712f96a3f9a1701b").unwrap());

        let expected_relay_transaction = RelayTransaction {
            chain_id,
            from: EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap()),
            signature,
            data,
            deadline: 0,
            gas_limit: 300000,
            compensation: 49999999999999999,
            relay_contract_address: EthAddress::from_slice(
                &hex::decode("9b4fa5a1d9f6812e2b56b36fbde62736fa82c2a7").unwrap(),
            ),
            to: EthAddress::from_slice(&hex::decode("53c2048dad4fcfab44c3ef3d16e882b5178df42b").unwrap()),
        };

        assert_eq!(relay_transaction, expected_relay_transaction);
    }

    #[test]
    fn should_serialize_deserialize_relay_tx_as_json() {
        // deserialize
        let json_str = r#"
            {
                "chainId": 3,
                "from": "0x736661736533BcfC9cc35649e6324aceFb7D32c1",
                "to": "0xFDE83bd51bddAA39F15c1Bf50E222a7AE5831D83",
                "data": "0xf15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000",
                "deadline": 0,
                "gasLimit": 100000,
                "compensation": "500000000",
                "relayContractAddress": "0x9b4FA5A1D9f6812e2B56B36fBde62736Fa82c2a7",
                "signature": "0x983fa37fae8243405e58dfa71a7f1aa01ce0d7f0658708d201825fb6b2e0280625b37f5000c3f74a96011ac50450b750d75ec7a5c634b815e2645af93bd2a4041c"
            }
        "#;

        let relay_transaction = RelayTransaction::from_str(json_str).unwrap();

        let chain_id = EthChainId::Ropsten;
        let eth_private_key =
            EthPrivateKey::from_str("841734cb439af03575c37c29b332619f3da9ea2fbaed58a1c8b1188ecff2a8dd").unwrap();
        let from = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000").unwrap();
        let deadline = Some(0);
        let gas_limit = 100000;
        let compensation = 500000000;
        let to = EthAddress::from_slice(&hex::decode("FDE83bd51bddAA39F15c1Bf50E222a7AE5831D83").unwrap());

        let expected_relay_transaction = RelayTransaction::new(
            from,
            &chain_id,
            eth_private_key,
            data,
            deadline,
            gas_limit,
            compensation,
            to,
        )
        .unwrap();

        assert_eq!(relay_transaction, expected_relay_transaction);

        // serialize
        let expected_relay_transaction = "{\"chainId\":3,\"from\":\"0x736661736533bcfc9cc35649e6324acefb7d32c1\",\"signature\":\"0x983fa37fae8243405e58dfa71a7f1aa01ce0d7f0658708d201825fb6b2e0280625b37f5000c3f74a96011ac50450b750d75ec7a5c634b815e2645af93bd2a4041c\",\"data\":\"0xf15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000\",\"deadline\":0,\"gasLimit\":100000,\"compensation\":\"500000000\",\"relayContractAddress\":\"0x9b4fa5a1d9f6812e2b56b36fbde62736fa82c2a7\",\"to\":\"0xfde83bd51bddaa39f15c1bf50e222a7ae5831d83\"}".to_string();
        let relay_transaction = relay_transaction.to_string().unwrap();

        assert_eq!(relay_transaction, expected_relay_transaction);
    }

    #[test]
    fn should_serialize_relay_tx_to_bytes() {
        let expected_result = hex::decode("f8f394fde83bd51bddaa39f15c1bf50e222a7ae5831d8394736661736533bcfc9cc35649e6324acefb7d32c1b864f15da72900000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004746573740000000000000000000000000000000000000000000000000000000080841dcd6500830186a003949b4fa5a1d9f6812e2b56b36fbde62736fa82c2a7b841983fa37fae8243405e58dfa71a7f1aa01ce0d7f0658708d201825fb6b2e0280625b37f5000c3f74a96011ac50450b750d75ec7a5c634b815e2645af93bd2a4041c").unwrap();
        let expected_tx_hash = "e1961095d0482c74d3018a90a7a5a2d7cb1fdac88f89cffbce3c7c638ee22294";
        let expected_tx_hex = "f8f394fde83bd51bddaa39f15c1bf50e222a7ae5831d8394736661736533bcfc9cc35649e6324acefb7d32c1b864f15da72900000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004746573740000000000000000000000000000000000000000000000000000000080841dcd6500830186a003949b4fa5a1d9f6812e2b56b36fbde62736fa82c2a7b841983fa37fae8243405e58dfa71a7f1aa01ce0d7f0658708d201825fb6b2e0280625b37f5000c3f74a96011ac50450b750d75ec7a5c634b815e2645af93bd2a4041c";

        let chain_id = EthChainId::Ropsten;
        let eth_private_key =
            EthPrivateKey::from_str("841734cb439af03575c37c29b332619f3da9ea2fbaed58a1c8b1188ecff2a8dd").unwrap();
        let from = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let data = hex::decode("f15da729000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000047465737400000000000000000000000000000000000000000000000000000000").unwrap();
        let deadline = Some(0);
        let gas_limit = 100000;
        let compensation = 500000000;
        let to = EthAddress::from_slice(&hex::decode("FDE83bd51bddAA39F15c1Bf50E222a7AE5831D83").unwrap());

        let relay_transaction = RelayTransaction::new(
            from,
            &chain_id,
            eth_private_key,
            data,
            deadline,
            gas_limit,
            compensation,
            to,
        )
        .unwrap();

        // bytes
        let result = relay_transaction.serialize_bytes();
        assert_eq!(result, expected_result);

        // hash
        let tx_hash = relay_transaction.get_tx_hash();
        assert_eq!(tx_hash, expected_tx_hash);

        // hex
        let tx_hex = relay_transaction.serialize_hex();
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
