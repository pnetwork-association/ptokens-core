use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    chains::eth::{
        eth_constants::{ETH_ADDRESS_SIZE_IN_BYTES, ETH_WORD_SIZE_IN_BYTES},
        eth_contracts::encode_fxn_call,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_log::EthLogExt,
    },
    metadata::metadata_chain_id::{MetadataChainId, METADATA_CHAIN_ID_NUMBER_OF_BYTES},
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};

const EMPTY_DATA: Bytes = vec![];

pub const ERC777_CHANGE_PNETWORK_GAS_LIMIT: usize = 30_000;
pub const ERC777_MINT_WITH_DATA_GAS_LIMIT: usize = 450_000;
pub const ERC777_MINT_WITH_NO_DATA_GAS_LIMIT: usize = 180_000;

const ERC777_CHANGE_PNETWORK_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"newPNetwork\",\"type\":\"address\"}],\"name\":\"changePNetwork\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"signature\":\"0xfd4add66\"}]";

const ERC777_MINT_WITH_NO_DATA_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

const ERC777_MINT_WITH_DATA_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"},{\"name\":\"userData\",\"type\":\"bytes\"},{\"name\":\"operatorData\",\"type\":\"bytes\"}],\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

pub const ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA_HEX: &str =
    "78e6c3f67f57c26578f2487b930b70d844bcc8dd8f4d629fb4af81252ab5aa65";

const ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA_HEX: &str =
    "4599e9bf0d45c505e011d0e11f473510f083a4fdc45e3f795d58bb5379dbad68";

const ERC777_REDEEM_EVENT_TOPIC_V2_HEX: &str = "dd56da0e6e7b301867b3632876d707f60c7cbf4b06f9ae191c67ea016cc5bf31";

lazy_static! {
    pub static ref ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA: EthHash = {
        EthHash::from_slice(
            &hex::decode(ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA_HEX)
                .expect("✘ Invalid hex in `ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA_HEX`"),
        )
    };
    pub static ref ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA: EthHash = {
        EthHash::from_slice(
            &hex::decode(ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA_HEX)
                .expect("✘ Invalid hex in `ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA_HEX`"),
        )
    };
    static ref ERC777_REDEEM_EVENT_TOPIC_V2: EthHash = {
        EthHash::from_slice(
            &hex::decode(ERC777_REDEEM_EVENT_TOPIC_V2_HEX)
                .expect("✘ Invalid hex in `ERC777_REDEEM_EVENT_TOPIC_V2_HEX`"),
        )
    };
}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
enum ERC777SupportedTopics {
    V2,
    V1WithUserData,
    V1WithoutUserData,
}

impl ERC777SupportedTopics {
    fn to_bytes(&self) -> Bytes {
        match &self {
            Self::V2 => ERC777_REDEEM_EVENT_TOPIC_V2.as_bytes().to_vec(),
            Self::V1WithUserData => ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA.as_bytes().to_vec(),
            Self::V1WithoutUserData => ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA.as_bytes().to_vec(),
        }
    }

    fn from_bytes(bytes: Bytes) -> Result<Self> {
        let result = Self::get_all()
            .iter()
            .zip(Self::get_all_as_bytes().iter())
            .filter(|(_, supported_topic_bytes)| bytes == supported_topic_bytes.to_vec())
            .map(|(supported_topic, _)| supported_topic)
            .cloned()
            .collect::<Vec<Self>>();
        if result.is_empty() {
            Err("Cannot get `ERC777SupportedTopics` from bytes - unrecognized topic!".into())
        } else {
            Ok(result[0].clone())
        }
    }

    fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }

    fn get_all_as_bytes() -> Vec<Bytes> {
        Self::get_all().iter().map(Self::to_bytes).collect()
    }

    fn from_topic(topic: &EthHash) -> Result<Self> {
        Self::from_bytes(topic.as_bytes().to_vec())
    }
}

pub fn encode_erc777_change_pnetwork_fxn_data(new_ptoken_address: EthAddress) -> Result<Bytes> {
    encode_fxn_call(ERC777_CHANGE_PNETWORK_ABI, "changePNetwork", &[EthAbiToken::Address(
        new_ptoken_address,
    )])
}

pub fn encode_erc777_mint_with_no_data_fxn(recipient: &EthAddress, value: &U256) -> Result<Bytes> {
    encode_fxn_call(ERC777_MINT_WITH_NO_DATA_ABI, "mint", &[
        EthAbiToken::Address(*recipient),
        EthAbiToken::Uint(*value),
    ])
}

fn encode_erc777_mint_with_data_fxn(
    recipient: &EthAddress,
    value: &U256,
    user_data: &[Byte],
    operator_data: &[Byte],
) -> Result<Bytes> {
    encode_fxn_call(ERC777_MINT_WITH_DATA_ABI, "mint", &[
        EthAbiToken::Address(*recipient),
        EthAbiToken::Uint(*value),
        EthAbiToken::Bytes(user_data.to_vec()),
        EthAbiToken::Bytes(operator_data.to_vec()),
    ])
}

fn get_eth_calldata_from_maybe_data(maybe_data: Option<Bytes>) -> Bytes {
    maybe_data.unwrap_or(EMPTY_DATA).to_vec()
}

pub fn encode_erc777_mint_fxn_maybe_with_data(
    recipient: &EthAddress,
    value: &U256,
    user_data: Option<Bytes>,
    operator_data: Option<Bytes>,
) -> Result<Bytes> {
    match user_data.is_some() | operator_data.is_some() {
        false => encode_erc777_mint_with_no_data_fxn(recipient, value),
        true => encode_erc777_mint_with_data_fxn(
            recipient,
            value,
            &get_eth_calldata_from_maybe_data(user_data),
            &get_eth_calldata_from_maybe_data(operator_data),
        ),
    }
}

pub fn get_signed_erc777_change_pnetwork_tx<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    new_address: EthAddress,
) -> Result<String> {
    const ZERO_ETH_VALUE: usize = 0;
    let nonce_before_incrementing = eth_db_utils.get_eth_account_nonce_from_db()?;
    eth_db_utils
        .increment_eth_account_nonce_in_db(1)
        .and(Ok(EthTransaction::new_unsigned(
            encode_erc777_change_pnetwork_fxn_data(new_address)?,
            nonce_before_incrementing,
            ZERO_ETH_VALUE,
            eth_db_utils.get_erc777_contract_address_from_db()?,
            &eth_db_utils.get_eth_chain_id_from_db()?,
            ERC777_CHANGE_PNETWORK_GAS_LIMIT,
            eth_db_utils.get_eth_gas_price_from_db()?,
        )
        .sign(&eth_db_utils.get_eth_private_key_from_db()?)?
        .serialize_hex()))
}

#[derive(Debug, Clone, Constructor, Eq, PartialEq)]
pub struct Erc777RedeemEvent {
    pub redeemer: EthAddress,
    pub value: U256,
    pub underlying_asset_recipient: String,
    pub user_data: Bytes,
    pub origin_chain_id: Option<MetadataChainId>,
    pub destination_chain_id: Option<MetadataChainId>,
}

impl Erc777RedeemEvent {
    fn get_err_msg(field: &str) -> String {
        format!("Error getting `{}` from `EthOnEvmErc777RedeemEvent`!", field)
    }

    fn from_v1_log_without_user_data<L: EthLogExt>(log: &L) -> Result<Self> {
        info!("✔ Attemping to get `Erc777RedeemEvent` from v1 log WITHOUT user data...");
        let tokens = eth_abi_decode(&[EthAbiParamType::Uint(256), EthAbiParamType::String], &log.get_data())?;
        log.check_has_x_topics(2).and_then(|_| {
            Ok(Self {
                user_data: vec![],
                origin_chain_id: None,
                destination_chain_id: None,
                redeemer: EthAddress::from_slice(
                    &log.get_topics()[1][ETH_WORD_SIZE_IN_BYTES - ETH_ADDRESS_SIZE_IN_BYTES..],
                ),
                value: match tokens[0] {
                    EthAbiToken::Uint(value) => Ok(value),
                    _ => Err(Self::get_err_msg("value")),
                }?,
                underlying_asset_recipient: match tokens[1] {
                    EthAbiToken::String(ref value) => Ok(value.clone()),
                    _ => Err(Self::get_err_msg("underlying_asset_recipient")),
                }?,
            })
        })
    }

    // TODO test!
    fn from_v1_log_with_user_data<L: EthLogExt>(log: &L) -> Result<Self> {
        info!("✔ Attemping to get `Erc777RedeemEvent` from v1 log WITH user data...");
        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::Bytes,
            ],
            &log.get_data(),
        )?;
        log.check_has_x_topics(2).and_then(|_| {
            Ok(Self {
                origin_chain_id: None,
                destination_chain_id: None,
                redeemer: EthAddress::from_slice(
                    &log.get_topics()[1][ETH_WORD_SIZE_IN_BYTES - ETH_ADDRESS_SIZE_IN_BYTES..],
                ),
                value: match tokens[0] {
                    EthAbiToken::Uint(value) => Ok(value),
                    _ => Err(Self::get_err_msg("value")),
                }?,
                underlying_asset_recipient: match tokens[1] {
                    EthAbiToken::String(ref value) => Ok(value.clone()),
                    _ => Err(Self::get_err_msg("underlying_asset_recipient")),
                }?,
                user_data: match tokens[2] {
                    EthAbiToken::Bytes(ref bytes) => Ok(bytes.to_vec()),
                    _ => Err(Self::get_err_msg("user_data")),
                }?,
            })
        })
    }

    fn from_v2_log<L: EthLogExt>(log: &L) -> Result<Self> {
        info!("✔ Attemping to get `Erc777RedeemEvent` from v2 log...");
        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Uint(256),
                EthAbiParamType::String,
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(METADATA_CHAIN_ID_NUMBER_OF_BYTES),
                EthAbiParamType::FixedBytes(METADATA_CHAIN_ID_NUMBER_OF_BYTES),
            ],
            &log.get_data(),
        )?;
        log.check_has_x_topics(2).and_then(|_| {
            Ok(Self {
                redeemer: EthAddress::from_slice(
                    &log.get_topics()[1][ETH_WORD_SIZE_IN_BYTES - ETH_ADDRESS_SIZE_IN_BYTES..],
                ),
                value: match tokens[0] {
                    EthAbiToken::Uint(value) => Ok(value),
                    _ => Err(Self::get_err_msg("value")),
                }?,
                underlying_asset_recipient: match tokens[1] {
                    EthAbiToken::String(ref value) => Ok(value.clone()),
                    _ => Err(Self::get_err_msg("underlying_asset_recipient")),
                }?,
                user_data: match tokens[2] {
                    EthAbiToken::Bytes(ref bytes) => Ok(bytes.to_vec()),
                    _ => Err(Self::get_err_msg("user_data")),
                }?,
                origin_chain_id: match tokens[3] {
                    EthAbiToken::FixedBytes(ref bytes) => Ok(Some(MetadataChainId::from_bytes(bytes)?)),
                    _ => Err(Self::get_err_msg("origin_chain_id")),
                }?,
                destination_chain_id: match tokens[4] {
                    EthAbiToken::FixedBytes(ref bytes) => Ok(Some(MetadataChainId::from_bytes(bytes)?)),
                    _ => Err(Self::get_err_msg("origin_chain_id")),
                }?,
            })
        })
    }

    pub fn from_eth_log<L: EthLogExt>(log: &L) -> Result<Self> {
        ERC777SupportedTopics::from_topic(&log.get_event_signature()?).and_then(|supported_topic| match supported_topic
        {
            ERC777SupportedTopics::V2 => Self::from_v2_log(log),
            ERC777SupportedTopics::V1WithUserData => Self::from_v1_log_with_user_data(log),
            ERC777SupportedTopics::V1WithoutUserData => Self::from_v1_log_without_user_data(log),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{
            eth_log::EthLog,
            eth_test_utils::{get_sample_log_with_erc20_peg_in_event, get_sample_log_with_erc777_redeem},
        },
        errors::AppError,
    };

    #[test]
    fn should_encode_erc777_change_pnetwork_fxn_data() {
        let expected_result = "fd4add66000000000000000000000000736661736533bcfc9cc35649e6324acefb7d32c1";
        let address = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let result = encode_erc777_change_pnetwork_fxn_data(address).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_encode_erc777_mint_with_no_data_fxn() {
        let expected_result = "40c10f190000000000000000000000001739624f5cd969885a224da84418d12b8570d61a0000000000000000000000000000000000000000000000000000000000000001";
        let recipient = EthAddress::from_slice(&hex::decode("1739624f5cd969885a224da84418d12b8570d61a").unwrap());
        let amount = U256::from_dec_str("1").unwrap();
        let result = encode_erc777_mint_with_no_data_fxn(&recipient, &amount).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_encode_erc777_mint_with_data_fxn() {
        let expected_result = "dcdc7dd00000000000000000000000001739624f5cd969885a224da84418d12b8570d61a0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000003decaff00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003c0ffee0000000000000000000000000000000000000000000000000000000000";
        let recipient = EthAddress::from_slice(&hex::decode("1739624f5cd969885a224da84418d12b8570d61a").unwrap());
        let amount = U256::from_dec_str("1").unwrap();
        let user_data = vec![0xde, 0xca, 0xff];
        let operator_data = vec![0xc0, 0xff, 0xee];
        let result = encode_erc777_mint_with_data_fxn(&recipient, &amount, &user_data, &operator_data).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_redeem_event_params_from_log_without_user_data() {
        let log = get_sample_log_with_erc777_redeem();
        let expected_result = Erc777RedeemEvent::new(
            EthAddress::from_slice(&hex::decode("edb86cd455ef3ca43f0e227e00469c3bdfa40628").unwrap()),
            U256::from_dec_str("6660000000000").unwrap(),
            "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            vec![],
            None,
            None,
        );
        let result_1 = Erc777RedeemEvent::from_v1_log_without_user_data(&log).unwrap();
        let result_2 = Erc777RedeemEvent::from_eth_log(&log).unwrap();
        assert_eq!(result_1, expected_result);
        assert_eq!(result_1, result_2);
    }

    #[test]
    fn should_fail_to_get_params_from_non_erc777_redeem_event() {
        let expected_error = "Cannot get `ERC777SupportedTopics` from bytes - unrecognized topic!";
        let log = get_sample_log_with_erc20_peg_in_event().unwrap();
        match Erc777RedeemEvent::from_eth_log(&log) {
            Ok(_) => panic!("Should not have succeeded"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_decode_v1_redeem_event_log_with_user_data() {
        let s = "{\"data\":\"000000000000000000000000000000000000000000000000000000000000029a000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000a616e2061646472657373000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006decaffc0ffee0000000000000000000000000000000000000000000000000000\",\"address\": \"Cf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9\",\"topics\":[\"4599e9bf0d45c505e011d0e11f473510f083a4fdc45e3f795d58bb5379dbad68\",\"000000000000000000000000976ea74026e726554db657fa54763abd0c3a0aa9\"]}";
        let log = EthLog::from_str(s).unwrap();
        let expected_result = Erc777RedeemEvent::new(
            EthAddress::from_slice(&hex::decode("976ea74026e726554db657fa54763abd0c3a0aa9").unwrap()),
            U256::from(666),
            "an address".to_string(),
            hex::decode("decaffc0ffee").unwrap(),
            None,
            None,
        );
        let result_1 = Erc777RedeemEvent::from_v1_log_with_user_data(&log).unwrap();
        let result_2 = Erc777RedeemEvent::from_eth_log(&log).unwrap();
        assert_eq!(result_1, expected_result);
        assert_eq!(result_1, result_2);
    }

    #[test]
    fn should_decode_v2_redeem_event_log() {
        let s = "{\"address\":\"0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9\",\"topics\":[\"0xdd56da0e6e7b301867b3632876d707f60c7cbf4b06f9ae191c67ea016cc5bf31\",\"0x000000000000000000000000976ea74026e726554db657fa54763abd0c3a0aa9\"],\"data\":\"0x000000000000000000000000000000000000000000000000000000000000029a00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e00069c3220000000000000000000000000000000000000000000000000000000000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a616e2061646472657373000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\"}";
        let log = EthLog::from_str(s).unwrap();
        let expected_result = Erc777RedeemEvent {
            user_data: vec![],
            value: U256::from(666),
            underlying_asset_recipient: "an address".to_string(),
            origin_chain_id: Some(MetadataChainId::EthereumRopsten),
            destination_chain_id: Some(MetadataChainId::EthereumRinkeby),
            redeemer: EthAddress::from_slice(&hex::decode("976ea74026e726554db657fa54763abd0c3a0aa9").unwrap()),
        };
        let result_1 = Erc777RedeemEvent::from_v2_log(&log).unwrap();
        let result_2 = Erc777RedeemEvent::from_eth_log(&log).unwrap();
        assert_eq!(result_1, expected_result);
        assert_eq!(result_1, result_2);
    }
}
