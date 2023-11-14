use log::Level;

use super::{initialize_file_logger, initialize_stdout_logger};
use crate::{LogConfig, SentinelError};

pub fn init_logger(config: &LogConfig, cli_log_level: Option<Level>) -> Result<(), SentinelError> {
    debug!("initializing logging...");
    if config.use_file_logging {
        initialize_file_logger(config, cli_log_level)
    } else {
        initialize_stdout_logger(config, cli_log_level)
    }
}
