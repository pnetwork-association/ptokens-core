use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("cannot find db at path: {0}")]
    NoDb(String),

    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("{0}")]
    Json(serde_json::Value),

    #[error("sentinel error: {0}")]
    Sentinel(#[from] lib::SentinelError),
}
