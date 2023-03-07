mod file_logger;
mod stdout_logger;

use std::result::Result;

use self::{file_logger::initialize_file_logger, stdout_logger::initialize_stdout_logger};
use crate::{config::LogConfig, SentinelError};

pub fn init_logger(config: &LogConfig) -> Result<(), SentinelError> {
    if config.use_file_logging {
        initialize_file_logger(config)
    } else {
        initialize_stdout_logger(config)
    }
}
