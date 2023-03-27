use common_eth::convert_hex_to_h256;
use ethereum_types::H256 as EthHash;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UserOperation {}

macro_rules! get_topics {
    ($($name:ident => $hex:expr),* $(,)?) => {
        $(
            lazy_static! {
                pub static ref $name: EthHash = convert_hex_to_h256(&$hex)
                    .expect(&format!("Converting from hex shouldn't fail for {}", stringify!($name)));
            }
        )*
    }
}

get_topics!(
    USER_OPERATION_TOPIC => "375102e6250006aa44e53e96d29b6a719df98a1c40b28c133e684ef40e52b989",
);

/*
#[derive(Clone, Debug, Default, Constructor, Serialize, Deserialize)]
pub struct AddressAndTopic {
    pub(crate) address: EthAddress,
    pub(crate) topic: EthHash,
}

pub trait AddressesAndTopicsT {
    fn from_config<C: ConfigT>(config: &C) -> Result<Self, SentinelError>
    where
        Self: Sized;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Constructor, IntoIterator)]
pub struct NativeAddressesAndTopics(Vec<AddressAndTopic>);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Constructor, IntoIterator)]
pub struct HostAddressesAndTopics(Vec<AddressAndTopic>);

impl AddressesAndTopicsT for NativeAddressesAndTopics {
    fn from_config<C: ConfigT>(config: &C) -> Result<Self, SentinelError> {
        if config.side().is_host() {
            return Err(SentinelError::Custom(
                "Cannot create native addresses and topics from host config!".into(),
            ));
        }
        let mut r: Vec<AddressAndTopic> = vec![];
        let addresses = config.get_contract_addresses();
        for address in addresses {
            r.push(AddressAndTopic::new(address, *NATIVE_PEG_IN_TOPIC));
            r.push(AddressAndTopic::new(address, *NATIVE_ERC20_TRANSFER_TOPIC));
        }
        Ok(Self::new(r))
    }
}

impl AddressesAndTopicsT for HostAddressesAndTopics {
    fn from_config<C: ConfigT>(config: &C) -> Result<Self, SentinelError> {
        if config.side().is_native() {
            return Err(SentinelError::Custom(
                "Cannot create host addresses and topics from host config!".into(),
            ));
        }
        let mut r: Vec<AddressAndTopic> = vec![];
        let addresses = config.get_contract_addresses();
        for address in addresses {
            r.push(AddressAndTopic::new(address, *HOST_MINTED_TOPIC));
            r.push(AddressAndTopic::new(address, *HOST_PEG_OUT_TOPIC));
        }
        Ok(Self::new(r))
    }
}
*/
