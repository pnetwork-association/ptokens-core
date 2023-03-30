#[derive(Debug)]
pub enum Error {
    NoBlock(u64),
    NoLatestBlock,
    Call(jsonrpsee::core::Error),
    PushTx(jsonrpsee::core::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Call(ref e) => write!(f, "could not make call: {e}"),
            Self::NoBlock(ref n) => write!(f, "could not get block {n}"),
            Self::NoLatestBlock => write!(f, "could not get latest block"),
            Self::PushTx(ref e) => write!(f, "could not push tx to endpoint: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;

        match self {
            NoBlock(_) | NoLatestBlock | PushTx(_) | Call(_) => None,
        }
    }
}
