#[macro_export]
macro_rules! get_debug_command_hash {
    ($($e:expr),*) => {
        // NOTE: We need to use a closure so that we can capture variables from the outer
        // environment.
        || -> $crate::types::Result<String> {
            use tiny_keccak::Hasher;
            let mut hasher = tiny_keccak::Keccak::v256();
            let mut hash = [0u8; $crate::constants::ETH_HASH_LENGTH];
            let bytes_to_hash = [$(serde_json::to_vec($e)?,)*].concat();
            hasher.update(&bytes_to_hash);
            hasher.finalize(&mut hash);
            Ok(format!("0x{}", hex::encode(&hash)))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    #[test]
    fn should_get_debug_command_hash_via_macro() {
        #[derive(Serialize)]
        struct SomeStruct {
            x: u64,
        }
        let result = get_debug_command_hash!("x", &SomeStruct { x: 1337 }, &true, &false)().unwrap();
        let expected_result = "0xdbf91dc346f60035958f07e14aef36e8115b75daf7e6756205b132c0ae2380df";
        assert_eq!(result, expected_result);
    }
}
