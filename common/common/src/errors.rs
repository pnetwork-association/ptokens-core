quick_error! {
    #[derive(Debug)]
    pub enum AppError {
        BlockAlreadyInDbError(err: crate::BlockAlreadyInDbError) {
            from()
            display("{err}")
        }
        NoParentError(err: crate::NoParentError) {
            from()
            display("{err}")
        }
        Json(err: serde_json::Value) {
            from()
            display("{}", err)
        }
        FromStrRadix(err: ethereum_types::FromStrRadixErr) {
            from()
            display("ethereum-types from_str_radix err: {}", err)
        }
        Custom(err: String) {
            from()
            from(err: &str) -> (err.into())
            display("✘ {}", err)
        }
        IoError(err: std::io::Error) {
            from()
            display("✘ I/O error: {}", err)
        }
        RmpDecodeError(err: rmp_serde::decode::Error) {
            from()
            display("✘ MsgPack decoding error: {}", err)
        }
        RmpEncodeError(err: rmp_serde::encode::Error) {
            from()
            display("✘ MsgPack encoding error: {}", err)
        }
        HexError(err: hex::FromHexError) {
            from()
            display("✘ Hex error: {}", err)
        }
        CryptoError(err: secp256k1::Error) {
            from()
            display("✘ Crypto error: {}", err)
        }
        BitcoinCryptoError(err: bitcoin::secp256k1::Error) {
            from()
            display("✘ Bitcoin crypto error: {}", err)
        }
        Base58Error(err: bitcoin::util::base58::Error) {
            from()
            display("✘ Base58 error: {}", err)
        }
        LitecoinBase58Error(err: litecoin::util::base58::Error) {
            from()
            display("litecoin base58 error: {}", err)
        }
        SerdeJsonError(err: serde_json::Error) {
            from()
            display("✘ Serde-Json error: {}", err)
        }
        FromUtf8Error(err: std::str::Utf8Error) {
            from()
            display("✘ From utf8 error: {}", err)
        }
        SetLoggerError(err: log::SetLoggerError) {
            from(log::SetLoggerError)
            display("✘ Error setting up logger: {}", err)
        }
        ParseIntError(err: std::num::ParseIntError) {
            from()
            display("✘ Parse Int error: {}", err)
        }
        ChronoError(err: chrono::ParseError) {
            from()
            display("✘ Chrono error: {}", err)
        }
        EosPrimitivesError(err: eos_chain::Error) {
            from()
            display("✘ EOS chain error: {:?}", err)
        }
        BitcoinHexError(err: bitcoin::hashes::hex::Error) {
            from()
            display("✘ Bitcoin hex error: {}", err)
        }
        SystemTimeError(err: std::time::SystemTimeError) {
            from()
            display("✘ System time error: {}", err)
        }
        FromSliceError(err: std::array::TryFromSliceError) {
            from(std::array::TryFromSliceError)
            display("✘ From slice error: {}", err)
        }
        BitcoinHashError(err: bitcoin::hashes::Error) {
            from()
            display("✘ Bitcoin hash error: {}", err)
        }
        BitcoinError(err: bitcoin::consensus::encode::Error) {
            from()
            display("✘ Bitcoin error: {}", err)
        }
        LitecoinEncodeError(err: litecoin::consensus::encode::Error) {
            from()
            display("litecoin encoding error: {}", err)
        }
        BitcoinAddressError(err: bitcoin::util::address::Error) {
            from()
            display("✘ Bitcoin address error: {}", err)
        }
        LitecoinAddressError(err: litecoin::util::address::Error) {
            from()
            display("litecoin address error: {}", err)
        }
        BitcoinScriptError(err: bitcoin::blockdata::script::Error) {
            from()
            display("✘ Bitcoin script error: {}", err)
        }
        BitcoinKeyError(err: bitcoin::util::key::Error) {
            from()
            display("✘ Bitcoin key error: {}", err)
        }
        LitecoinKeyError(err: litecoin::util::key::Error) {
            from()
            display("litecoin key error: {}", err)
        }
        LitecoinError(err: litecoin::Error) {
            from()
            display("litecoin error: {}", err)
        }
        EosPrimitivesNamesError(err: eos_chain::ParseNameError) {
            from()
            display("✘ EOS chain names error: {}", err)
        }
        EthAbiError(err: ethabi::Error) {
            from()
            display("✘ ETH ABI error: {}", err)
        }
        RlpDecoderError(err: rlp::DecoderError) {
            from()
            display("✘ RLP decoder error: {}", err)
        }
        FromDecStrErr(err: ethereum_types::FromDecStrErr) {
            from()
            display("✘ Ethereum types `from_dec_str` err: {}", err)
        }
        EosParseAssetErr(err: eos_chain::ParseAssetError) {
            from()
            display("✘ EOS parse asset error: {:?}", err)
        }
        EosWriteError(err: eos_chain::WriteError) {
            from()
            display("✘ EOS write error: {:?}", err)
        }
        TryFromError(err: std::num::TryFromIntError) {
            from()
            display("✘ `TryFrom` error: {:?}", err)
        }
        AlgorandError(err: rust_algorand::AlgorandError) {
            from()
            display("✘ Algorand error: {:?}", err)
        }
        TryFromSliceError(err: std::array::TryFromSliceError) {
            from()
            display("✘ `TryFromSlice` error: {:?}", err)
        }
        ParseFloatError(err: std::num::ParseFloatError) {
            from()
            display("✘ `ParseFloatError` error: {:?}", err)
        }
        Base64DecodeError(err: base64::DecodeError) {
            from()
            display("✘ Base64 decoder error: {}", err)
        }
        RustCHexError(err: rustc_hex::FromHexError) {
            from()
            display("✘ Rustc hex error: {}", err)
        }
        RecoveryError(err: web3::signing::RecoveryError) {
            from()
            display("✘ Web3 signature recovery error: {}", err)
        }
        EIP712Error(err: eip_712::Error) {
            from()
            display("✘ EIP712 error: {}", err)
        }
        DocoptError(err: docopt::Error) {
            from()
            display("Docopt error: {}", err)
        }
        NoneError(err: &'static str) {
            display("✘ None error {}", err)
        }
    }
}
