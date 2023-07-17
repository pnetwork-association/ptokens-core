use std::{env, result::Result, str::FromStr};

use common_eth::EthPrivateKey;
use dotenv::dotenv;

use super::SentinelError;

const BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX: &str = "_BROADCASTER_PRIVATE_KEY";

pub struct Env {}

impl Env {
    pub fn init() -> Result<(), SentinelError> {
        if let Err(e) = dotenv() {
            error!("dotenv error {e}");
            Err(SentinelError::Custom(
                "could not initialize dotenv - does an `.env` file exist?".into(),
            ))
        } else {
            Ok(())
        }
    }

    fn get_env_var(s: &str) -> Result<String, SentinelError> {
        env::var(s).map_err(|_| SentinelError::MissingEnvVar(s.into()))
    }

    fn get_eth_pk_from_env_var(s: &str) -> Result<EthPrivateKey, SentinelError> {
        Ok(EthPrivateKey::from_str(&Self::get_env_var(s)?)?)
    }

    pub fn get_native_broadcaster_private_key() -> Result<EthPrivateKey, SentinelError> {
        Self::get_eth_pk_from_env_var(&format!("NATIVE{BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX}"))
    }

    pub fn get_host_broadcaster_private_key() -> Result<EthPrivateKey, SentinelError> {
        Self::get_eth_pk_from_env_var(&format!("HOST{BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX}"))
    }
}
