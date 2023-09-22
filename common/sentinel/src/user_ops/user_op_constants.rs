use common_eth::convert_hex_to_h256;
use ethereum_types::H256 as EthHash;

macro_rules! get_topics {
    ($($name:ident => $hex:expr),* $(,)?) => {
        $(
            lazy_static! {
                pub static ref $name: EthHash = convert_hex_to_h256(&$hex)
                    .expect(&format!("converting from hex should not fail for {}", stringify!($name)));
            }
        )*
    }
}

get_topics!(
    // NOTE: This is fired when `userSend` is called
    WITNESSED_USER_OP_TOPIC => "f4faec7e493ced73194f78a54c931da9a2d6c6b9552b223cc9ad2965322789b7",
    ENQUEUED_USER_OP_TOPIC => "e7bf22971bde3dd8a6a3bf8434e8b7a7c7554dad8328f741da1484d67b445c19",
    EXECUTED_USER_OP_TOPIC => "0dd9442ca0ceb76d843508ae85c58c2ef3742491a1cc480e4c0d1c96ab9965a6",
    CANCELLED_USER_OP_TOPIC => "0x2c4c3f1ebc7e7a6c814ed2315a9e1ef863749841a858f5c27437ecf53ca8b39f",
);

pub const USER_OP_CANCEL_TX_GAS_LIMIT: u64 = 2_000_000; // FIXME should be in config!
