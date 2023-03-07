#[derive(Debug, PartialEq)]
pub enum Error {
    NoBlock(u64),
    NoLatestBlock,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::NoBlock(ref n) => write!(f, "could not get block {n}"),
            Self::NoLatestBlock => write!(f, "could not get latest block"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;

        match self {
            NoBlock(_) | NoLatestBlock => None,
        }
    }
}
