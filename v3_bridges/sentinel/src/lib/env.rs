use std::{env, result::Result, str::FromStr};

use common_eth::EthPrivateKey;
use dotenv::dotenv;
use thiserror::Error;

const BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX: &str = "_BROADCASTER_PRIVATE_KEY";

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("cannot find `.env` file")]
    MissingEnvFile,

    #[error("missing env var: '{0}'")]
    MissingEnvVar(String),

    #[error("bad private key for env var: '{0}'")]
    MalformedPrivateKey(String),
}

pub struct Env {}

impl Env {
    pub fn init() -> Result<(), EnvError> {
        if let Err(e) = dotenv() {
            error!("dotenv error {e}");
            Err(EnvError::MissingEnvFile)
        } else {
            Ok(())
        }
    }

    fn get_env_var(s: &str) -> Result<String, EnvError> {
        env::var(s).map_err(|_| EnvError::MissingEnvVar(s.into()))
    }

    fn get_eth_pk_from_env_var(s: &str) -> Result<EthPrivateKey, EnvError> {
        EthPrivateKey::from_str(&Self::get_env_var(s)?).map_err(|e| {
            error!("{e}");
            EnvError::MalformedPrivateKey(s.into())
        })
    }

    pub fn get_native_broadcaster_private_key() -> Result<EthPrivateKey, EnvError> {
        Self::get_eth_pk_from_env_var(&format!("NATIVE{BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX}"))
    }

    pub fn get_host_broadcaster_private_key() -> Result<EthPrivateKey, EnvError> {
        Self::get_eth_pk_from_env_var(&format!("HOST{BROADCASTER_PRIVATE_KEY_ENV_VAR_SUFFIX}"))
    }
}
