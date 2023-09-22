pub const HEX_RADIX: u32 = 16;
pub const DEFAULT_SLEEP_TIME: u64 = 15_000;
pub const MILLISECONDS_MULTIPLIER: u64 = 1000;

lazy_static! {
    pub static ref HOST_PROTOCOL_ID: crate::ProtocolId = crate::ProtocolId::Ethereum;
    pub static ref NATIVE_PROTOCOL_ID: crate::ProtocolId = crate::ProtocolId::Ethereum;
}
