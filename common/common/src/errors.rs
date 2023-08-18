use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    BlockAlreadyInDbError(crate::BlockAlreadyInDbError),

    #[error("{0}")]
    NoParentError(crate::NoParentError),

    #[error("✘ {0}")]
    Custom(String),

    #[error("{}", 0.to_string())]
    Json(serde_json::Value),

    #[error("ethereum-types from_str_radix err: {0}")]
    FromStrRadix(#[from] ethereum_types::FromStrRadixErr),

    #[error("✘ I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("✘ MsgPack decoding error: {0}")]
    RmpDecodeError(#[from] rmp_serde::decode::Error),

    #[error("✘ MsgPack encoding error: {0}")]
    RmpEncodeError(#[from] rmp_serde::encode::Error),

    #[error("✘ Hex error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("✘ Crypto error: {0}")]
    CryptoError(#[from] secp256k1::Error),

    #[error("✘ Bitcoin crypto error: {0}")]
    BitcoinCryptoError(#[from] bitcoin::secp256k1::Error),

    #[error("✘ bitcoin base58 error: {0}")]
    Base58Error(#[from] bitcoin::util::base58::Error),

    #[error("✘ Serde-Json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("✘ From utf8 error: {0}")]
    FromUtf8Error(#[from] std::str::Utf8Error),

    #[error("✘ Error setting up logger: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),

    #[error("✘ Parse Int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("✘ Chrono error: {0}")]
    ChronoError(#[from] chrono::ParseError),

    #[error("✘ EOS chain error: {0:?}")]
    EosPrimitivesError(String),

    #[error("✘ Bitcoin hex error: {0}")]
    BitcoinHexError(#[from] bitcoin::hashes::hex::Error),

    #[error("✘ System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("✘ From slice error: {0}")]
    FromSliceError(#[from] std::array::TryFromSliceError),

    #[error("✘ Bitcoin hash error: {0}")]
    BitcoinHashError(#[from] bitcoin::hashes::Error),

    #[error("bitcoin consensus error: {0}")]
    BitcoinError(#[from] bitcoin::consensus::encode::Error),

    #[error("bitcoin address error: {0}")]
    BitcoinAddressError(#[from] bitcoin::util::address::Error),

    #[error("✘ Bitcoin script error: {0}")]
    BitcoinScriptError(#[from] bitcoin::blockdata::script::Error),

    #[error("bitcoin key error: {0}")]
    BitcoinKeyError(#[from] bitcoin::util::key::Error),

    #[error("✘ EOS chain names error: {0}")]
    EosPrimitivesNamesError(String),

    #[error("✘ ETH ABI error: {0}")]
    EthAbiError(#[from] ethabi::Error),

    #[error("✘ RLP decoder error: {0}")]
    RlpDecoderError(#[from] rlp::DecoderError),

    #[error("✘ Ethereum types `from_dec_str` #[from] {0}")]
    FromDecStrErr(#[from] ethereum_types::FromDecStrErr),

    #[error("✘ EOS parse asset error: {0:?}")]
    EosParseAssetErr(eos_chain::ParseAssetError),

    #[error("✘ EOS parse name error: {0:?}")]
    EosParseNameErr(eos_chain::ParseNameError),

    #[error("✘ EOS write error: {0:?}")]
    EosWriteError(eos_chain::WriteError),

    #[error("✘ `TryFrom` error: {0:?}")]
    TryFromError(#[from] std::num::TryFromIntError),

    #[error("✘ Algorand error: {0:?}")]
    AlgorandError(#[from] rust_algorand::AlgorandError),

    #[error("✘ `ParseFloatError` error: {0:?}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("✘ Rustc hex error: {0}")]
    RustCHexError(#[from] rustc_hex::FromHexError),

    #[error("✘ Web3 signature recovery error: {0}")]
    RecoveryError(#[from] web3::signing::RecoveryError),

    #[error("✘ EIP712 error: {0}")]
    EIP712(eip_712::Error),

    #[error("Docopt error: {0}")]
    DocoptError(#[from] docopt::Error),

    #[error("✘ none error {0}")]
    NoneError(&'static str),

    #[error("eos chain error: {}", 0.to_string())]
    EosChain(eos_chain::Error),

    #[error("litecoin base58 error: {0}")]
    LitecoinBase58Error(#[from] litecoin::util::base58::Error),

    #[error("litecoin consensus error: {0}")]
    LitecoinEncodeError(#[from] litecoin::consensus::encode::Error),

    #[error("litecoin address error: {0}")]
    LitecoinAddressError(#[from] litecoin::util::address::Error),

    #[error("litecoin key error: {0}")]
    LitecoinKeyError(#[from] litecoin::util::key::Error),

    #[error("litecoin error: {0}")]
    LitecoinError(#[from] litecoin::Error),
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::from(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Custom(s)
    }
}

impl From<eos_chain::ParseNameError> for AppError {
    fn from(e: eos_chain::ParseNameError) -> Self {
        Self::EosParseNameErr(e)
    }
}

impl From<eos_chain::Error> for AppError {
    fn from(e: eos_chain::Error) -> Self {
        Self::EosChain(e)
    }
}

impl From<eip_712::Error> for AppError {
    fn from(e: eip_712::Error) -> Self {
        Self::EIP712(e)
    }
}

impl From<eos_chain::WriteError> for AppError {
    fn from(e: eos_chain::WriteError) -> Self {
        Self::EosWriteError(e)
    }
}

impl From<serde_json::Value> for AppError {
    fn from(e: serde_json::Value) -> Self {
        Self::Json(e)
    }
}

impl From<eos_chain::ParseAssetError> for AppError {
    fn from(e: eos_chain::ParseAssetError) -> Self {
        Self::EosParseAssetErr(e)
    }
}
