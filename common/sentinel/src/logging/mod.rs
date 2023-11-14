mod file_logger;
mod init_logger;
mod log_level;
mod stdout_logger;

use self::{file_logger::initialize_file_logger, stdout_logger::initialize_stdout_logger};
pub use self::{init_logger::init_logger, log_level::LogLevel};
