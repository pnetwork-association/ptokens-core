use common_eth::convert_hex_to_h256;
use ethereum_types::H256 as EthHash;

pub const HEX_RADIX: u32 = 16;
pub const MILLISECONDS_MULTIPLIER: u64 = 1000;

// NOTE: Originally we worked w/ > 1 topic, hence using a macro - bit overkill now.
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
    USER_OPERATION_TOPIC => "ba98a314fb19bf102109515e22a4e48acbbe8f5610a657a9ed6cb3327afbc2e2",
);
