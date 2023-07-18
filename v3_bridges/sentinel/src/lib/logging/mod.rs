mod file_logger;
mod log_level;
mod stdout_logger;

use std::result::Result;

use log::Level;

pub use self::log_level::LogLevel;
use self::{file_logger::initialize_file_logger, stdout_logger::initialize_stdout_logger};
use crate::{config::LogConfig, SentinelError};

pub fn init_logger(config: &LogConfig, cli_log_level: Option<Level>) -> Result<(), SentinelError> {
    if config.use_file_logging {
        initialize_file_logger(config, cli_log_level)
    } else {
        initialize_stdout_logger(config, cli_log_level)
    }
}
