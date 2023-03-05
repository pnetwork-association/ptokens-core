mod file_logger;
mod stdout_logger;

use anyhow::Result;

use self::{file_logger::initialize_file_logger, stdout_logger::initialize_stdout_logger};
use crate::config::LogConfig;

pub fn init_logger(config: &LogConfig) -> Result<()> {
    if config.use_file_logging {
        initialize_file_logger(config)
    } else {
        initialize_stdout_logger(config)
    }
}
