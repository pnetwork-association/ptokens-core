use std::result::Result;

use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming};

use crate::{config::LogConfig, SentinelError};

pub fn initialize_file_logger(config: &LogConfig) -> Result<(), SentinelError> {
    let num_logs_to_keep = config.max_num_logs / 2; // NOTE: We'll keep half of them compressed.

    Logger::try_with_str(config.level.as_str()).and_then(|logger| {
        logger
            .format(flexi_logger::with_thread) // NOTE: This adds more detail to log entries, timestamp, file-path & thread etc.
            .log_to_file(FileSpec::default().directory(&config.path))
            .rotate(
                Criterion::Size(config.max_log_size),
                Naming::Timestamps,
                Cleanup::KeepLogAndCompressedFiles(num_logs_to_keep, num_logs_to_keep),
            )
            .append()
            .start()
    })?;
    info!("File logger initialized!");
    Ok(())
}
