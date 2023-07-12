use std::{env, str::FromStr};

use common_eth::EthPrivateKey;

use crate::SentinelError;

const BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX: &str = "_BROADCASTER_PRIVATE_KEY";

fn get_env_var(s: &str) -> Result<String, SentinelError> {
    env::var(s).map_err(|_| SentinelError::MissingEnvVar(s.into()))
}

fn get_eth_pk_from_env_var(s: &str) -> Result<EthPrivateKey, SentinelError> {
    Ok(EthPrivateKey::from_str(&get_env_var(s)?)?)
}

pub fn get_native_broadcaster_private_key_from_env() -> Result<EthPrivateKey, SentinelError> {
    get_eth_pk_from_env_var(&format!("NATIVE{BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX}"))
}

pub fn get_host_broadcaster_private_key_from_env() -> Result<EthPrivateKey, SentinelError> {
    get_eth_pk_from_env_var(&format!("HOST{BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX}"))
}
