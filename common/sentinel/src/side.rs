use clap::ValueEnum;
use common::BridgeSide;

// NOTE: Used as a proxy for common::BridgeSide so we don't have to derive ValueEnum from that.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Side {
    /// native
    Native,
    /// host
    Host,
}

impl From<Side> for BridgeSide {
    fn from(val: Side) -> Self {
        match val {
            Side::Host => BridgeSide::Host,
            Side::Native => BridgeSide::Native,
        }
    }
}
