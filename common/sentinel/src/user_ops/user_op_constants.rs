use common_eth::convert_hex_to_h256;
use ethereum_types::H256 as EthHash;

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
    WITNESSED_USER_OP_TOPIC => "ba98a314fb19bf102109515e22a4e48acbbe8f5610a657a9ed6cb3327afbc2e2",
    ENQUEUED_USER_OP_TOPIC => "d1a85d51ecfea5edd75f97fcf615b22c6f56eaf8f0487db9fadfbe661689b9af",
    EXECUTED_USER_OP_TOPIC => "fb83c807750a326c5845536dc89b4d2da9f1f5e0df344e9f69f27c84f4d7d726",
    CANCELLED_USER_OP_TOPIC => "ec5d8f38737ebccaa579d2caeaed8fbc5f2c7c598fee1eb335429c8c48ec2598",
);

pub const USER_OP_CANCEL_TX_GAS_LIMIT: u64 = 2_000_000;
