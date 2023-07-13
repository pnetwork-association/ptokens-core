use clap::ValueEnum;
use log::Level;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    /// info
    Info,
    /// warn
    Warn,
    /// error
    Error,
    /// debug
    Debug,
    /// trace
    Trace,
}

impl From<LogLevel> for Level {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Info => Level::Info,
            LogLevel::Warn => Level::Warn,
            LogLevel::Error => Level::Error,
            LogLevel::Debug => Level::Debug,
            LogLevel::Trace => Level::Trace,
        }
    }
}
