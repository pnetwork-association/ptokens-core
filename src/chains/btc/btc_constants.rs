#[cfg(test)] // NOTE Because of real BTC tx test-vectors
pub const PTOKEN_P2SH_SCRIPT_BYTES: usize = 0;

#[cfg(not(test))]
pub const PTOKEN_P2SH_SCRIPT_BYTES: usize = 101;

pub const BTC_TAIL_LENGTH: u64 = 10;
pub const MAX_NUM_OUTPUTS: usize = 2;
pub const BTC_PUB_KEY_SLICE_LENGTH: usize = 33;
pub const MINIMUM_REQUIRED_SATOSHIS: u64 = 5000;
pub const DEFAULT_BTC_SEQUENCE: u32 = 4_294_967_295; // NOTE: 0xFFFFFFFF
pub const BTC_CORE_IS_INITIALIZED_JSON: &str = "{btc_enclave_initialized:true}";

// NOTE: Following is used as placeholder for bad address parsing in ETH params!
pub const PLACEHOLDER_BTC_ADDRESS: &str = "msTgHeQgPZ11LRcUdtfzagEfiZyKF57DhR";

create_db_keys_and_json!(
    "Btc";
    "BTC_FEE_KEY" => "btc-fee-key",
    "BTC_ADDRESS_KEY" => "btc-address",
    "BTC_DIFFICULTY" => "btc-difficulty",
    "BTC_NETWORK_KEY" => "btc-network-key",
    "BTC_LINKER_HASH_KEY" => "btc-linker-hash",
    "BTC_PRIVATE_KEY_DB_KEY" => "btc-private-key",
    "PTOKEN_GENESIS_HASH_KEY" => "provable-ptoken",
    "BTC_CANON_BLOCK_HASH_KEY" => "btc-canon-block",
    "BTC_LATEST_BLOCK_HASH_KEY" => "btc-latest-block",
    "BTC_ANCHOR_BLOCK_HASH_KEY" => "btc-anchor-block",
    "BTC_ACCOUNT_NONCE_KEY" => "btc-account-nonce-key",
    "BTC_PUBLIC_KEY_DB_KEY" => "btc-public-key-db-key",
    "BTC_TAIL_BLOCK_HASH_KEY" => "btc-tail-block-hash-key",
    "BTC_CANON_TO_TIP_LENGTH_KEY" => "btc-canon-to-tip-length"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btc_db_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = BtcDatabaseKeysJson {
            BTC_ACCOUNT_NONCE_KEY:
                "48236d034b7d7fac3b4550bdbe5682eb012d1717bb345c39c5add04be5139880".to_string(),
            BTC_ADDRESS_KEY:
                "bdf6e75595f2a65ce048e0416b8c2a8462288116db886b551b2891adceb0a53a".to_string(),
            BTC_ANCHOR_BLOCK_HASH_KEY:
                "bb005e5d49d23fc16c62b7971672f0f44043866cf19e4aa2d77db7f9632d0d83".to_string(),
            BTC_CANON_BLOCK_HASH_KEY:
                "ed228247ba940027aa9406ef39c2aa07f650bfa53f0b8478f2d90836615912b8".to_string(),
            BTC_CANON_TO_TIP_LENGTH_KEY:
                "2d9b6327983926c2dd9636f3c8bc13b811af80858c08fe1b9d019ebdcf73049c".to_string(),
            BTC_DIFFICULTY:
                "0ed532c16cd0bcc543cdcd01132c38349fd25e85b2d7f4609b66943bc8500a7c".to_string(),
            BTC_FEE_KEY:
                "6ded8f6cf1097edaf81e815dec1810946dd32327ecdc9de506ca7d1535c34801".to_string(),
            BTC_LATEST_BLOCK_HASH_KEY:
                "22f781fdf51ac53605f603b9abeaddd618d29eb7ebed285a919abf128379a0a2".to_string(),
            BTC_LINKER_HASH_KEY:
                "98e63aa8f93943b3bfea2ee4d0e063942415618cfc0cd51828de4de7b4698039".to_string(),
            BTC_NETWORK_KEY:
                "f2321e29a0792487edd90debfc9a85fcb39856a5343801e794c5c915aa341ee8".to_string(),
            BTC_PRIVATE_KEY_DB_KEY:
                "d8c4da823c79e9245163a8db18b7e9d6107f7487e624a4db9bdc3acb788902de".to_string(),
            BTC_PUBLIC_KEY_DB_KEY:
                "ee7ec6657db53cd1d8055d61bf00ff615063701493ede450dc5c31132ae6cfd1".to_string(),
            BTC_TAIL_BLOCK_HASH_KEY:
                "26ab99d609131225d7ecf087632b5b6771468931273d0f6c16b09c9bbe316f71".to_string(),
            PTOKEN_GENESIS_HASH_KEY:
                "7eb2e65416dd107602495454d1ed094ae475cff2f3bfb2e2ae68a1c52bc0d66f".to_string(),
        };
        let result = BtcDatabaseKeysJson::new();
        assert_eq!(result, expected_result);
    }
}
