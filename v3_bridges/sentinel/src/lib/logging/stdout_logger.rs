use anyhow::Result;
use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming};

use crate::config::LogConfig;

pub fn initialize_stdout_logger(config: &LogConfig) -> Result<()> {
    Logger::try_with_str(config.level.as_str()).and_then(|logger| {
        logger
            .format(flexi_logger::colored_with_thread) // NOTE: This adds more detail to log entries, timestamp, file-path & thread etc.
            .log_to_stdout()
            .append()
            .start()
    })?;
    info!("Stdout Logger initialized!");
    Ok(())
}
