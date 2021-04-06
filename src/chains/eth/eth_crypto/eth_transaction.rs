use std::fs;

use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, U256};
use rlp::RlpStream;

use crate::{
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_chain_id::EthChainId,
        eth_constants::{
            GAS_LIMIT_FOR_MINTING_TX,
            GAS_LIMIT_FOR_PTOKEN_DEPLOY,
            VALUE_FOR_MINTING_TX,
            VALUE_FOR_PTOKEN_DEPLOY,
        },
        eth_contracts::erc777::encode_erc777_mint_fxn_maybe_with_data,
        eth_crypto::eth_private_key::EthPrivateKey,
        eth_traits::{EthSigningCapabilities, EthTxInfoCompatible},
        eth_types::{EthSignature, EthSignedTransaction},
        eth_utils::strip_new_line_chars,
    },
    types::{Byte, Bytes, Result},
};

#[derive(Debug, Clone, Eq, PartialEq, Deref, Constructor)]
pub struct EthTransactions(pub Vec<EthTransaction>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EthTransaction {
    pub v: u64,
    pub r: U256,
    pub s: U256,
    pub to: Bytes,
    pub nonce: U256,
    pub value: U256,
    pub data: Bytes,
    pub chain_id: Byte,
    pub gas_limit: U256,
    pub gas_price: U256,
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

    pub fn new_contract(
        data: Bytes,
        nonce: u64,
        value: usize,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
    ) -> EthTransaction {
        Self::new_eth_tx(vec![], data, nonce, value, chain_id, gas_limit, gas_price)
    }

    fn new_eth_tx(
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
            v: chain_id.to_byte().into(), // Per EIP155
            nonce: nonce.into(),
            value: value.into(),
            chain_id: chain_id.to_byte(),
            gas_limit: gas_limit.into(),
            gas_price: gas_price.into(),
        }
    }

    fn add_signature_to_transaction(mut self, sig: EthSignature) -> Self {
        self.r = sig[0..32].into();
        self.s = sig[32..64].into();
        self.v = Self::calculate_v_from_chain_id(sig[64], self.chain_id);
        self
    }

    fn calculate_v_from_chain_id(sig_v: u8, chain_id: u8) -> u64 {
        chain_id as u64 * 2 + sig_v as u64 + 35 // Per EIP155
    }

    pub fn sign<T: EthSigningCapabilities>(self, pk: &T) -> Result<Self> {
        pk.sign_message_bytes(&self.serialize_bytes())
            .map(|sig| self.add_signature_to_transaction(sig))
    }

    pub fn serialize_hex(&self) -> String {
        hex::encode(self.serialize_bytes())
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

pub fn get_ptoken_smart_contract_bytecode(path: &str) -> Result<Bytes> {
    info!("✔ Getting ETH smart-contract bytecode...");
    let contents = match fs::read_to_string(path) {
        Ok(file) => Ok(file),
        Err(err) => Err(format!(
            "✘ Cannot find ETH smart-contract byte code at: '{}'\n✘ {}\n{}",
            path, err, "✘ Maybe look into the pToken ERC777 bytecode generator tool?",
        )),
    }?;
    Ok(hex::decode(strip_new_line_chars(contents))?)
}

fn get_unsigned_ptoken_smart_contract_tx(
    nonce: u64,
    chain_id: &EthChainId,
    gas_price: u64,
    bytecode_path: &str,
) -> Result<EthTransaction> {
    Ok(EthTransaction::new_contract(
        get_ptoken_smart_contract_bytecode(&bytecode_path)?,
        nonce,
        VALUE_FOR_PTOKEN_DEPLOY,
        chain_id,
        GAS_LIMIT_FOR_PTOKEN_DEPLOY,
        gas_price,
    ))
}

pub fn get_signed_ptoken_smart_contract_tx(
    nonce: u64,
    chain_id: &EthChainId,
    eth_private_key: &EthPrivateKey,
    gas_price: u64,
    bytecode_path: &str,
) -> Result<EthSignedTransaction> {
    Ok(
        get_unsigned_ptoken_smart_contract_tx(nonce, chain_id, gas_price, bytecode_path)?
            .sign(eth_private_key)?
            .serialize_hex(),
    )
}

pub fn get_unsigned_minting_tx(
    nonce: u64,
    amount: &U256,
    chain_id: &EthChainId,
    to: EthAddress,
    gas_price: u64,
    recipient: &EthAddress,
    user_data: Option<&[Byte]>,
    operator_data: Option<&[Byte]>,
) -> Result<EthTransaction> {
    Ok(EthTransaction::new_unsigned(
        encode_erc777_mint_fxn_maybe_with_data(recipient, amount, user_data, operator_data)?,
        nonce,
        VALUE_FOR_MINTING_TX,
        to,
        chain_id,
        GAS_LIMIT_FOR_MINTING_TX,
        gas_price,
    ))
}

pub fn get_signed_minting_tx(
    amount: &U256,
    nonce: u64,
    chain_id: &EthChainId,
    to: EthAddress,
    gas_price: u64,
    recipient: &EthAddress,
    eth_private_key: &EthPrivateKey,
    user_data: Option<&[Byte]>,
    operator_data: Option<&[Byte]>,
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
    use crate::chains::eth::eth_test_utils::{
        get_sample_eth_address,
        get_sample_eth_private_key,
        get_sample_unsigned_eth_transaction,
        ETH_SMART_CONTRACT_BYTECODE_PATH,
    };

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
    fn should_read_smart_contract_bytecode_from_file() {
        if let Err(e) = get_ptoken_smart_contract_bytecode(&ETH_SMART_CONTRACT_BYTECODE_PATH.to_string()) {
            panic!("Errored reading bytecode file: {}", e);
        }
    }

    #[test]
    fn should_get_unsigned_eth_smart_contract_transaction() {
        let nonce = 1;
        let chain_id = EthChainId::Rinkeby;
        let gas_price = 20_000_000_000;
        if let Err(e) = get_unsigned_ptoken_smart_contract_tx(
            nonce,
            &chain_id,
            gas_price,
            &ETH_SMART_CONTRACT_BYTECODE_PATH.to_string(),
        ) {
            panic!("Errored getting unsigned ETH s-c tx: {}", e);
        }
    }

    #[test]
    fn should_get_signed_eth_smart_contract_tx() {
        let nonce = 16;
        let chain_id = EthChainId::Rinkeby;
        let gas_price = 20_000_000_000;
        let eth_private_key = get_sample_eth_private_key();
        let result = get_signed_ptoken_smart_contract_tx(
            nonce,
            &chain_id,
            &eth_private_key,
            gas_price,
            &ETH_SMART_CONTRACT_BYTECODE_PATH.to_string(),
        )
        .unwrap();
        // NOTE: Real tx broadcast here: https://ropsten.etherscan.io/tx/0xe618338e2344546305096a360d0c796892e1554c5af0097e08595bca8144fc83
        let expected_result = "f93038108504a817c800833d09008080b92fe560806040523480156200001157600080fd5b5060405162002ec538038062002ec5833981018060405260608110156200003757600080fd5b8101908080516401000000008111156200005057600080fd5b820160208101848111156200006457600080fd5b81516401000000008111828201871017156200007f57600080fd5b505092919060200180516401000000008111156200009c57600080fd5b82016020810184811115620000b057600080fd5b8151640100000000811182820187101715620000cb57600080fd5b50509291906020018051640100000000811115620000e857600080fd5b82016020810184811115620000fc57600080fd5b81518560208202830111640100000000821117156200011a57600080fd5b5050855190935085925084915083906200013c90600290602086019062000391565b5081516200015290600390602085019062000391565b5080516200016890600490602084019062000416565b5060005b600454811015620001ca576001600560006004848154811015156200018d57fe5b6000918252602080832091909101546001600160a01b031683528201929092526040019020805460ff19169115159190911790556001016200016c565b50604080517f455243373737546f6b656e0000000000000000000000000000000000000000008152815190819003600b0181207f29965a1d0000000000000000000000000000000000000000000000000000000082523060048301819052602483019190915260448201529051731820a4b7618bde71dce8cdc73aab6c95905fad24916329965a1d91606480830192600092919082900301818387803b1580156200027457600080fd5b505af115801562000289573d6000803e3d6000fd5b5050604080517f4552433230546f6b656e000000000000000000000000000000000000000000008152815190819003600a0181207f29965a1d0000000000000000000000000000000000000000000000000000000082523060048301819052602483019190915260448201529051731820a4b7618bde71dce8cdc73aab6c95905fad2493506329965a1d9250606480830192600092919082900301818387803b1580156200033657600080fd5b505af11580156200034b573d6000803e3d6000fd5b50505050505050620003626200038c60201b60201c565b600980546001600160a01b0319166001600160a01b039290921691909117905550620004c0915050565b335b90565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f10620003d457805160ff191683800117855562000404565b8280016001018555821562000404579182015b8281111562000404578251825591602001919060010190620003e7565b50620004129291506200047c565b5090565b8280548282559060005260206000209081019282156200046e579160200282015b828111156200046e57825182546001600160a01b0319166001600160a01b0390911617825560209092019160019091019062000437565b506200041292915062000499565b6200038e91905b8082111562000412576000815560010162000483565b6200038e91905b80821115620004125780546001600160a01b0319168155600101620004a0565b6129f580620004d06000396000f3fe608060405234801561001057600080fd5b50600436106101735760003560e01c8063959b8c3f116100de578063d95b637111610097578063fad8b32a11610071578063fad8b32a14610a39578063fc673c4f14610a5f578063fd4add6614610b9d578063fe9d930314610bc357610173565b8063d95b63711461089f578063dcdc7dd0146108cd578063dd62ed3e14610a0b57610173565b8063959b8c3f1461063857806395d89b411461065e5780639bd9bbc614610666578063a9059cbb1461071f578063ca16814e1461074b578063ce67c0031461076f57610173565b806324b76fd51161013057806324b76fd514610402578063313ce5671461047757806340c10f1914610495578063556f0dc7146104c157806362ad1b83146104c957806370a082311461061257610173565b806306e485381461017857806306fdde03146101d0578063095ea7b31461024d57806318160ddd1461028d5780631e9cee74146102a757806323b872dd146103cc575b600080fd5b610180610c6e565b60408051602080825283518183015283519192839290830191858101910280838360005b838110156101bc5781810151838201526020016101a4565b505050509050019250505060405180910390f35b6101d8610cd0565b6040805160208082528351818301528351919283929083019185019080838360005b838110156102125781810151838201526020016101fa565b50505050905090810190601f16801561023f5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6102796004803603604081101561026357600080fd5b506001600160a01b038135169060200135610d5a565b604080519115158252519081900360200190f35b610295610d7c565b60408051918252519081900360200190f35b6103ca600480360360a08110156102bd57600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b8111156102ec57600080fd5b8201836020820111156102fe57600080fd5b803590602001918460018302840111600160201b8311171561031f57600080fd5b919390929091602081019035600160201b81111561033c57600080fd5b82018360208201111561034e57600080fd5b803590602001918460018302840111600160201b8311171561036f57600080fd5b919390929091602081019035600160201b81111561038c57600080fd5b82018360208201111561039e57600080fd5b803590602001918460018302840111600160201b831117156103bf57600080fd5b509092509050610d82565b005b610279600480360360608110156103e257600080fd5b506001600160a01b03813581169160208101359091169060400135610ecc565b6102796004803603604081101561041857600080fd5b81359190810190604081016020820135600160201b81111561043957600080fd5b82018360208201111561044b57600080fd5b803590602001918460018302840111600160201b8311171561046c57600080fd5b509092509050611059565b61047f6110ab565b6040805160ff9092168252519081900360200190f35b610279600480360360408110156104ab57600080fd5b506001600160a01b0381351690602001356110b0565b6102956110dc565b6103ca600480360360a08110156104df57600080fd5b6001600160a01b03823581169260208101359091169160408201359190810190608081016060820135600160201b81111561051957600080fd5b82018360208201111561052b57600080fd5b803590602001918460018302840111600160201b8311171561054c57600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295949360208101935035915050600160201b81111561059e57600080fd5b8201836020820111156105b057600080fd5b803590602001918460018302840111600160201b831117156105d157600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295506110e1945050505050565b6102956004803603602081101561062857600080fd5b50356001600160a01b0316611150565b6103ca6004803603602081101561064e57600080fd5b50356001600160a01b031661116b565b6101d86112ba565b6103ca6004803603606081101561067c57600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b8111156106ab57600080fd5b8201836020820111156106bd57600080fd5b803590602001918460018302840111600160201b831117156106de57600080fd5b91908080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525092955061131b945050505050565b6102796004803603604081101561073557600080fd5b506001600160a01b03813516906020013561134d565b61075361142b565b604080516001600160a01b039092168252519081900360200190f35b6103ca6004803603606081101561078557600080fd5b81359190810190604081016020820135600160201b8111156107a657600080fd5b8201836020820111156107b857600080fd5b803590602001918460018302840111600160201b831117156107d957600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295949360208101935035915050600160201b81111561082b57600080fd5b82018360208201111561083d57600080fd5b803590602001918460018302840111600160201b8311171561085e57600080fd5b91908080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525092955061143a945050505050565b610279600480360360408110156108b557600080fd5b506001600160a01b0381358116916020013516611514565b610279600480360360808110156108e357600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b81111561091257600080fd5b82018360208201111561092457600080fd5b803590602001918460018302840111600160201b8311171561094557600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295949360208101935035915050600160201b81111561099757600080fd5b8201836020820111156109a957600080fd5b803590602001918460018302840111600160201b831117156109ca57600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295506115b6945050505050565b61029560048036036040811015610a2157600080fd5b506001600160a01b0381358116916020013516611678565b6103ca60048036036020811015610a4f57600080fd5b50356001600160a01b03166116a3565b6103ca60048036036080811015610a7557600080fd5b6001600160a01b0382351691602081013591810190606081016040820135600160201b811115610aa457600080fd5b820183602082011115610ab657600080fd5b803590602001918460018302840111600160201b83111715610ad757600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295949360208101935035915050600160201b811115610b2957600080fd5b820183602082011115610b3b57600080fd5b803590602001918460018302840111600160201b83111715610b5c57600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295506117ec945050505050565b6103ca60048036036020811015610bb357600080fd5b50356001600160a01b0316611857565b6103ca60048036036040811015610bd957600080fd5b81359190810190604081016020820135600160201b811115610bfa57600080fd5b820183602082011115610c0c57600080fd5b803590602001918460018302840111600160201b83111715610c2d57600080fd5b91908080601f0160208091040260200160405190810160405280939291908181526020018383808284376000920191909152509295506118d5945050505050565b60606004805480602002602001604051908101604052809291908181526020018280548015610cc657602002820191906000526020600020905b81546001600160a01b03168152600190910190602001808311610ca8575b5050505050905090565b60028054604080516020601f6000196101006001871615020190941685900493840181900481028201810190925282815260609390929091830182828015610cc65780601f10610d2e57610100808354040283529160200191610cc6565b820191906000526020600020905b815481529060010190602001808311610d3c57509395945050505050565b600080610d65611903565b9050610d72818585611907565b5060019392505050565b60015490565b610d93610d8d611903565b89611514565b1515610dd357604051600160e51b62461bcd02815260040180806020018281038252602c815260200180612909602c913960400191505060405180910390fd5b610e50610dde611903565b898989898080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525050604080516020601f8d018190048102820181019092528b815292508b91508a90819084018382808284376000920191909152506119b392505050565b876001600160a01b03167f78e6c3f67f57c26578f2487b930b70d844bcc8dd8f4d629fb4af81252ab5aa6588848460405180848152602001806020018281038252848482818152602001925080828437600083820152604051601f909101601f1916909201829003965090945050505050a25050505050505050565b60006001600160a01b0383161515610f1857604051600160e51b62461bcd0281526004018080602001828103825260248152602001806128e56024913960400191505060405180910390fd5b6001600160a01b0384161515610f6257604051600160e51b62461bcd02815260040180806020018281038252602681526020018061295e6026913960400191505060405180910390fd5b6000610f6c611903565b9050610f9a818686866040518060200160405280600081525060405180602001604052806000815250611be5565b610fc6818686866040518060200160405280600081525060405180602001604052806000815250611e30565b611020858261101b86604051806060016040528060298152602001612935602991396001600160a01b03808c166000908152600860209081526040808320938b1683529290522054919063ffffffff61204916565b611907565b61104e81868686604051806020016040528060008152506040518060200160405280600081525060006120e3565b506001949350505050565b6000610d72846040518060200160405280600081525085858080601f01602080910402602001604051908101604052809392919081815260200183838082843760009201919091525061143a92505050565b601290565b6000610d72838360405180602001604052806000815250604051806020016040528060008152506115b6565b600190565b6110f26110ec611903565b86611514565b151561113257604051600160e51b62461bcd02815260040180806020018281038252602c815260200180612909602c913960400191505060405180910390fd5b61114961113d611903565b86868686866001612389565b5050505050565b6001600160a01b031660009081526020819052604090205490565b806001600160a01b031661117d611903565b6001600160a01b031614156111c657604051600160e51b62461bcd0281526004018080602001828103825260248152602001806127d56024913960400191505060405180910390fd5b6001600160a01b03811660009081526005602052604090205460ff161561122957600760006111f3611903565b6001600160a01b03908116825260208083019390935260409182016000908120918516815292529020805460ff19169055611270565b600160066000611237611903565b6001600160a01b03908116825260208083019390935260409182016000908120918616815292529020805460ff19169115159190911790555b611278611903565b6001600160a01b0316816001600160a01b03167ff4caeb2d6ca8932a215a353d0703c326ec2d81fc68170f320eb2ab49e9df61f960405160405180910390a350565b60038054604080516020601f6002600019610100600188161502019095169490940493840181900481028201810190925282815260609390929091830182828015610cc65780601f10610d2e57610100808354040283529160200191610cc6565b611348611326611903565b61132e611903565b858585604051806020016040528060008152506001612389565b505050565b60006001600160a01b038316151561139957604051600160e51b62461bcd0281526004018080602001828103825260248152602001806128e56024913960400191505060405180910390fd5b60006113a3611903565b90506113d1818286866040518060200160405280600081525060405180602001604052806000815250611be5565b6113fd818286866040518060200160405280600081525060405180602001604052806000815250611e30565b610d7281828686604051806020016040528060008152506040518060200160405280600081525060006120e3565b6009546001600160a01b031681565b611464611445611903565b61144d611903565b8585604051806020016040528060008152506119b3565b336001600160a01b03167f78e6c3f67f57c26578f2487b930b70d844bcc8dd8f4d629fb4af81252ab5aa6584836040518083815260200180602001828103825283818151815260200191508051906020019080838360005b838110156114d45781810151838201526020016114bc565b50505050905090810190601f1680156115015780820380516001836020036101000a031916815260200191505b50935050505060405180910390a2505050565b6000816001600160a01b0316836001600160a01b0316148061157f57506001600160a01b03831660009081526005602052604090205460ff16801561157f57506001600160a01b0380831660009081526007602090815260408083209387168352929052205460ff16155b806115af57506001600160a01b0380831660009081526006602090815260408083209387168352929052205460ff165b9392505050565b6009546000906001600160a01b03166115cd611903565b6001600160a01b03161461161557604051600160e51b62461bcd0281526004018080602001828103825260228152602001806128216022913960400191505060405180910390fd5b6001600160a01b038516151561165f57604051600160e51b62461bcd0281526004018080602001828103825260288152602001806127f96028913960400191505060405180910390fd5b60095461104e906001600160a01b03168686868661245e565b6001600160a01b03918216600090815260086020908152604080832093909416825291909152205490565b6116ab611903565b6001600160a01b03828116911614156116f857604051600160e51b62461bcd0281526004018080602001828103825260218152602001806128436021913960400191505060405180910390fd5b6001600160a01b03811660009081526005602052604090205460ff161561176457600160076000611727611903565b6001600160a01b03908116825260208083019390935260409182016000908120918616815292529020805460ff19169115159190911790556117a2565b60066000611770611903565b6001600160a01b03908116825260208083019390935260409182016000908120918516815292529020805460ff191690555b6117aa611903565b6001600160a01b0316816001600160a01b03167f50546e66e5f44d728365dc3908c63bc5cfeeab470722c1677e3073a6ac294aa160405160405180910390a350565b6117fd6117f7611903565b85611514565b151561183d57604051600160e51b62461bcd02815260040180806020018281038252602c815260200180612909602c913960400191505060405180910390fd5b611851611848611903565b858585856119b3565b50505050565b6009546001600160a01b031661186b611903565b6001600160a01b0316146118b357604051600160e51b62461bcd0281526004018080602001828103825260348152602001806128646034913960400191505060405180910390fd5b600980546001600160a01b0319166001600160a01b0392909216919091179055565b6118ff6118e0611903565b6118e8611903565b8484604051806020016040528060008152506119b3565b5050565b3390565b6001600160a01b038216151561195157604051600160e51b62461bcd0281526004018080602001828103825260238152602001806129a76023913960400191505060405180910390fd5b6001600160a01b03808416600081815260086020908152604080832094871680845294825291829020859055815185815291517f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9259281900390910190a3505050565b6001600160a01b03841615156119fd57604051600160e51b62461bcd0281526004018080602001828103825260228152602001806127b36022913960400191505060405180910390fd5b611a0c85856000868686611be5565b611a4f83604051806060016040528060238152602001612984602391396001600160a01b038716600090815260208190526040902054919063ffffffff61204916565b6001600160a01b038516600090815260208190526040902055600154611a7b908463ffffffff61268e16565b600181905550836001600160a01b0316856001600160a01b03167fa78a9be3a7b862d26933ad85fb11d80ef66b8f972d7cbba06621d583943a4098858585604051808481526020018060200180602001838103835285818151815260200191508051906020019080838360005b83811015611b00578181015183820152602001611ae8565b50505050905090810190601f168015611b2d5780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b83811015611b60578181015183820152602001611b48565b50505050905090810190601f168015611b8d5780820380516001836020036101000a031916815260200191505b509550505050505060405180910390a36040805184815290516000916001600160a01b038716917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9181900360200190a35050505050565b60408051600160e11b63555ddc650281526001600160a01b03871660048201527f29ddb589b1fb5fc7cf394961c1adf5f8c6454761adf795e67fe149f658abe89560248201529051600091731820a4b7618bde71dce8cdc73aab6c95905fad249163aabbb8ca91604480820192602092909190829003018186803b158015611c6c57600080fd5b505afa158015611c80573d6000803e3d6000fd5b505050506040513d6020811015611c9657600080fd5b505190506001600160a01b03811615611e2757806001600160a01b03166375ab97828888888888886040518763ffffffff1660e01b815260040180876001600160a01b03166001600160a01b03168152602001866001600160a01b03166001600160a01b03168152602001856001600160a01b03166001600160a01b031681526020018481526020018060200180602001838103835285818151815260200191508051906020019080838360005b83811015611d5c578181015183820152602001611d44565b50505050905090810190601f168015611d895780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b83811015611dbc578181015183820152602001611da4565b50505050905090810190601f168015611de95780820380516001836020036101000a031916815260200191505b5098505050505050505050600060405180830381600087803b158015611e0e57600080fd5b505af1158015611e22573d6000803e3d6000fd5b505050505b50505050505050565b611e738360405180606001604052806027815260200161278c602791396001600160a01b038816600090815260208190526040902054919063ffffffff61204916565b6001600160a01b038087166000908152602081905260408082209390935590861681522054611ea8908463ffffffff6126d016565b600080866001600160a01b03166001600160a01b0316815260200190815260200160002081905550836001600160a01b0316856001600160a01b0316876001600160a01b03167f06b541ddaa720db2b10a4d0cdac39b8d360425fc073085fac19bc82614677987868686604051808481526020018060200180602001838103835285818151815260200191508051906020019080838360005b83811015611f59578181015183820152602001611f41565b50505050905090810190601f168015611f865780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b83811015611fb9578181015183820152602001611fa1565b50505050905090810190601f168015611fe65780820380516001836020036101000a031916815260200191505b509550505050505060405180910390a4836001600160a01b0316856001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef856040518082815260200191505060405180910390a3505050505050565b600081848411156120db57604051600160e51b62461bcd0281526004018080602001828103825283818151815260200191508051906020019080838360005b838110156120a0578181015183820152602001612088565b50505050905090810190601f1680156120cd5780820380516001836020036101000a031916815260200191505b509250505060405180910390fd5b505050900390565b60408051600160e11b63555ddc650281526001600160a01b03871660048201527fb281fc8c12954d22544db45de3159a39272895b169a852b314f9cc762e44c53b60248201529051600091731820a4b7618bde71dce8cdc73aab6c95905fad249163aabbb8ca91604480820192602092909190829003018186803b15801561216a57600080fd5b505afa15801561217e573d6000803e3d6000fd5b505050506040513d602081101561219457600080fd5b505190506001600160a01b0381161561232857806001600160a01b03166223de298989898989896040518763ffffffff1660e01b815260040180876001600160a01b03166001600160a01b03168152602001866001600160a01b03166001600160a01b03168152602001856001600160a01b03166001600160a01b031681526020018481526020018060200180602001838103835285818151815260200191508051906020019080838360005b83811015612259578181015183820152602001612241565b50505050905090810190601f1680156122865780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b838110156122b95781810151838201526020016122a1565b50505050905090810190601f1680156122e65780820380516001836020036101000a031916815260200191505b5098505050505050505050600060405180830381600087803b15801561230b57600080fd5b505af115801561231f573d6000803e3d6000fd5b5050505061237f565b811561237f57612340866001600160a01b031661272d565b1561237f57604051600160e51b62461bcd02815260040180806020018281038252604d815260200180612898604d913960600191505060405180910390fd5b5050505050505050565b6001600160a01b03861615156123d357604051600160e51b62461bcd02815260040180806020018281038252602281526020018061276a6022913960400191505060405180910390fd5b6001600160a01b03851615156124335760408051600160e51b62461bcd02815260206004820181905260248201527f4552433737373a2073656e6420746f20746865207a65726f2061646472657373604482015290519081900360640190fd5b612441878787878787611be5565b61244f878787878787611e30565b611e27878787878787876120e3565b6001600160a01b03841615156124be5760408051600160e51b62461bcd02815260206004820181905260248201527f4552433737373a206d696e7420746f20746865207a65726f2061646472657373604482015290519081900360640190fd5b6001546124d1908463ffffffff6126d016565b6001556001600160a01b0384166000908152602081905260409020546124fd908463ffffffff6126d016565b6001600160a01b03851660009081526020819052604081209190915561252a9086908686868660016120e3565b836001600160a01b0316856001600160a01b03167f2fe5be0146f74c5bce36c0b80911af6c7d86ff27e89d5cfa61fc681327954e5d858585604051808481526020018060200180602001838103835285818151815260200191508051906020019080838360005b838110156125a9578181015183820152602001612591565b50505050905090810190601f1680156125d65780820380516001836020036101000a031916815260200191505b50838103825284518152845160209182019186019080838360005b838110156126095781810151838201526020016125f1565b50505050905090810190601f1680156126365780820380516001836020036101000a031916815260200191505b509550505050505060405180910390a36040805184815290516001600160a01b038616916000917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9181900360200190a35050505050565b60006115af83836040518060400160405280601e81526020017f536166654d6174683a207375627472616374696f6e206f766572666c6f770000815250612049565b6000828201838110156115af5760408051600160e51b62461bcd02815260206004820152601b60248201527f536166654d6174683a206164646974696f6e206f766572666c6f770000000000604482015290519081900360640190fd5b6000813f7fc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a47081811480159061276157508115155b94935050505056fe4552433737373a2073656e642066726f6d20746865207a65726f20616464726573734552433737373a207472616e7366657220616d6f756e7420657863656564732062616c616e63654552433737373a206275726e2066726f6d20746865207a65726f20616464726573734552433737373a20617574686f72697a696e672073656c66206173206f70657261746f7270546f6b656e3a2043616e6e6f74206d696e7420746f20746865207a65726f2061646472657373214f6e6c792074686520704e6574776f726b2063616e206d696e7420746f6b656e73214552433737373a207265766f6b696e672073656c66206173206f70657261746f724f6e6c792074686520704e6574776f726b2063616e206368616e6765207468652060704e6574776f726b60206163636f756e74214552433737373a20746f6b656e20726563697069656e7420636f6e747261637420686173206e6f20696d706c656d656e74657220666f7220455243373737546f6b656e73526563697069656e744552433737373a207472616e7366657220746f20746865207a65726f20616464726573734552433737373a2063616c6c6572206973206e6f7420616e206f70657261746f7220666f7220686f6c6465724552433737373a207472616e7366657220616d6f756e74206578636565647320616c6c6f77616e63654552433737373a207472616e736665722066726f6d20746865207a65726f20616464726573734552433737373a206275726e20616d6f756e7420657863656564732062616c616e63654552433737373a20617070726f766520746f20746865207a65726f2061646472657373a165627a7a72305820a55dec8d756976f206e67ef62b604e7395bc3f0b815ade8e5c197066ec930aa30029000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000670546f6b656e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000450544b4e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac2ca0bc2180b426a1b0e216ad406536275f1b69e83cba2bc64e8755a52596bd95739aa052589b5d04199fb2de5e3ba655aee86daf10c51cb0b944edb0da6d6ac730621c".to_string();
        assert_eq!(result, expected_result);
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
