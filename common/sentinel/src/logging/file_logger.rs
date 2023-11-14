use std::result::Result;

use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming};
use log::Level;

use crate::{config::LogConfig, SentinelError};

pub fn initialize_file_logger(config: &LogConfig, cli_log_level: Option<Level>) -> Result<(), SentinelError> {
    let log_str = if let Some(l) = cli_log_level {
        l.as_str()
    } else {
        config.level.as_str()
    };
    let num_logs_to_keep = config.max_num_logs / 2; // NOTE: We'll keep half of them compressed.

    Logger::try_with_str(log_str).and_then(|logger| {
        logger
            .format(flexi_logger::colored_opt_format) // NOTE: This adds more detail to log entries, timestamp, file-path etc.
            .log_to_file(FileSpec::default().directory(&config.path))
            .rotate(
                Criterion::Size(config.max_log_size),
                Naming::Timestamps,
                Cleanup::KeepLogAndCompressedFiles(num_logs_to_keep, num_logs_to_keep),
            )
            .append()
            .start()
    })?;
    debug!("file logger initialized!");
    Ok(())
}
