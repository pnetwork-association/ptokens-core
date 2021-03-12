#![allow(dead_code)] // FIXME: rm!

use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    chains::eth::{eth_contracts::encode_fxn_call, eth_log::EthLog},
    types::{Bytes, Result},
};

pub const PERC20_PEGOUT_GAS_LIMIT: usize = 180_000;
pub const PERC20_MIGRATE_GAS_LIMIT: usize = 6_000_000;
pub const PERC20_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT: usize = 100_000;

pub const PERC20_ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenRecipient\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_tokenAmount\",\"type\":\"uint256\"}],\"name\":\"pegOut\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"addresspayable\",\"name\":\"_to\",\"type\":\"address\"}],\"name\":\"migrate\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"}],\"name\":\"addSupportedToken\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"SUCCESS\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"}],\"name\":\"removeSupportedToken\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"SUCCESS\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

pub fn encode_perc20_peg_out_fxn_data(
    recipient: EthAddress,
    token_contract_address: EthAddress,
    amount: U256,
) -> Result<Bytes> {
    encode_fxn_call(PERC20_ABI, "pegOut", &[
        EthAbiToken::Address(recipient),
        EthAbiToken::Address(token_contract_address),
        EthAbiToken::Uint(amount),
    ])
}

pub fn encode_perc20_migrate_fxn_data(migrate_to: EthAddress) -> Result<Bytes> {
    encode_fxn_call(PERC20_ABI, "migrate", &[EthAbiToken::Address(migrate_to)])
}

pub fn encode_perc20_add_supported_token_fx_data(token_to_support: EthAddress) -> Result<Bytes> {
    encode_fxn_call(PERC20_ABI, "addSupportedToken", &[EthAbiToken::Address(
        token_to_support,
    )])
}

pub fn encode_perc20_remove_supported_token_fx_data(token_to_remove: EthAddress) -> Result<Bytes> {
    encode_fxn_call(PERC20_ABI, "removeSupportedToken", &[EthAbiToken::Address(
        token_to_remove,
    )])
}

#[derive(Debug, PartialEq, Constructor)]
pub struct Perc20PegInEventParams {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub token_address: EthAddress,
    pub destination_address: String,
}

impl Perc20PegInEventParams {
    fn get_err_msg(field: &str) -> String {
        format!("Error getting `{}` from `Perc20PegInEvent`!", field)
    }

    fn from_log_with_user_data(log: &EthLog) -> Result<Self> {
        let tokens = eth_abi_decode(
            &vec![
                EthAbiParamType::Address,
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::Bytes,
            ],
            &log.data,
        )?;
        Ok(Self {
            token_address: match tokens[0] {
                EthAbiToken::Address(value) => Ok(value),
                _ => Err(Self::get_err_msg("token_address").to_string()),
            }?,
            token_sender: match tokens[1] {
                EthAbiToken::Address(value) => Ok(value),
                _ => Err(Self::get_err_msg("token_sender").to_string()),
            }?,
            token_amount: match tokens[2] {
                EthAbiToken::Uint(value) => Ok(value),
                _ => Err(Self::get_err_msg("token_amount").to_string()),
            }?,
            destination_address: match tokens[3] {
                EthAbiToken::String(ref value) => Ok(value.to_string()),
                _ => Err(Self::get_err_msg("destination_address").to_string()),
            }?,
            user_data: match tokens[4] {
                EthAbiToken::Bytes(ref value) => Ok(value.clone()),
                _ => Err(Self::get_err_msg("user_data").to_string()),
            }?,
        })
    }

    fn from_log_without_user_data(log: &EthLog) -> Result<Self> {
        let tokens = eth_abi_decode(
            &vec![
                EthAbiParamType::Address,
                EthAbiParamType::Address,
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
            ],
            &log.data,
        )?;
        Ok(Self {
            user_data: vec![],
            token_address: match tokens[0] {
                EthAbiToken::Address(value) => Ok(value),
                _ => Err(Self::get_err_msg("token_address").to_string()),
            }?,
            token_sender: match tokens[1] {
                EthAbiToken::Address(value) => Ok(value),
                _ => Err(Self::get_err_msg("token_sender").to_string()),
            }?,
            token_amount: match tokens[2] {
                EthAbiToken::Uint(value) => Ok(value),
                _ => Err(Self::get_err_msg("token_amount").to_string()),
            }?,
            destination_address: match tokens[3] {
                EthAbiToken::String(ref value) => Ok(value.to_string()),
                _ => Err(Self::get_err_msg("destination_address").to_string()),
            }?,
        })
    }

    pub fn from_log(log: &EthLog) -> Result<Self> {
        match Self::from_log_with_user_data(log) {
            Ok(res) => Ok(res),
            Err(_) => Self::from_log_without_user_data(log),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_test_utils::{get_sample_eth_address, get_sample_log_with_erc20_peg_in_event_2};

    #[test]
    fn should_encode_peg_out_fxn_data() {
        let amount = U256::from(1337);
        let recipient_address =
            EthAddress::from_slice(&hex::decode("edB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap());
        let token_address = EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap());
        let expected_result = "83c09d42000000000000000000000000edb86cd455ef3ca43f0e227e00469c3bdfa40628000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac0000000000000000000000000000000000000000000000000000000000000539";
        let result = hex::encode(encode_perc20_peg_out_fxn_data(recipient_address, token_address, amount).unwrap());
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_encode_migrate_fxn_data() {
        let address = EthAddress::from_slice(&hex::decode("edB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap());
        let expected_result = "ce5494bb000000000000000000000000edb86cd455ef3ca43f0e227e00469c3bdfa40628";
        let result = hex::encode(encode_perc20_migrate_fxn_data(address).unwrap());
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_encode_perc20_add_supported_token_fx_data() {
        let expected_result = "6d69fcaf0000000000000000000000001739624f5cd969885a224da84418d12b8570d61a";
        let address = get_sample_eth_address();
        let result = encode_perc20_add_supported_token_fx_data(address).unwrap();
        assert_eq!(hex::encode(&result), expected_result);
    }

    #[test]
    fn should_encode_perc20_remove_supported_token_fx_data() {
        let expected_result = "763191900000000000000000000000001739624f5cd969885a224da84418d12b8570d61a";
        let address = get_sample_eth_address();
        let result = encode_perc20_remove_supported_token_fx_data(address).unwrap();
        assert_eq!(hex::encode(&result), expected_result);
    }

    #[test]
    fn should_get_per20_peg_in_event_params_from_log_without_user_data() {
        let log = get_sample_log_with_erc20_peg_in_event_2().unwrap();
        let result = Perc20PegInEventParams::from_log(&log).unwrap();
        let expected_result = Perc20PegInEventParams {
            user_data: vec![],
            token_amount: U256::from_dec_str("50000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("7344d31d7025f72bd1d3c08645fa6b12d406fc05").unwrap()),
            token_address: EthAddress::from_slice(&hex::decode("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()),
            destination_address: "whateverxxxx".to_string(),
        };
        assert_eq!(result, expected_result);
    }
}
