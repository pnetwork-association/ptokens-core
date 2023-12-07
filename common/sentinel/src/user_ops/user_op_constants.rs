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
    WITNESSED_USER_OP_TOPIC => "9ffdc0f54ae0fe23916f708fc14e8278ee1077aa72e783808fae55caa865304a", // NOTE: This is fired when `userSend` is called
    ENQUEUED_USER_OP_TOPIC => "e7bf22971bde3dd8a6a3bf8434e8b7a7c7554dad8328f741da1484d67b445c19",
    EXECUTED_USER_OP_TOPIC => "0dd9442ca0ceb76d843508ae85c58c2ef3742491a1cc480e4c0d1c96ab9965a6",
    CANCELLED_USER_OP_TOPIC => "0x33fe909c76b8ce2d80c623608e768bdb2c69f1d53f55d56d0e562a6e9c567288",
);

pub const USER_OP_CANCEL_TX_GAS_LIMIT: u64 = 2_000_000; // FIXME should be in config!
