use std::result::Result;

use flexi_logger::Logger;
use log::Level;

use crate::{sentinel_config::LogConfig, SentinelError};

pub fn initialize_stdout_logger(config: &LogConfig, cli_args_log_level: Option<Level>) -> Result<(), SentinelError> {
    let log_str = if let Some(l) = cli_args_log_level {
        l.as_str()
    } else {
        config.level.as_str()
    };

    Logger::try_with_str(log_str).and_then(|logger| {
        logger
            .format(flexi_logger::colored_opt_format) // NOTE: This adds more detail to log entries, timestamp, file-path etc.
            .log_to_stdout()
            .append()
            .start()
    })?;
    debug!("Stdout Logger initialized!");
    Ok(())
}
