//! Command-line argument parsing definitions using `clap`.

pub use clap::{Parser, ValueEnum};
use kyotox_core::logger;

/// Command-line arguments for the Kyotox application.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandCli {
    /// Sets the application log level verbosity.
    #[arg(long, short='l', value_enum, default_value_t = LogLevel::Trace)]
    pub log: LogLevel,
}

/// Log verbosity levels available via command-line configuration.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default, Debug)]
pub enum LogLevel {
    /// Trace level logging (most verbose).
    Trace,
    /// Debug level logging.
    Debug,
    /// Informational level logging.
    Info,
    /// Warning level logging.
    Warn,
    /// Error level logging.
    Error,
    /// Logging disabled (default).
    #[default]
    Off,
}
impl Into<logger::LevelFilter> for LogLevel {
    fn into(self) -> logger::LevelFilter {
        match self {
            Self::Off => logger::LevelFilter::Off,
            Self::Debug => logger::LevelFilter::Debug,
            Self::Error => logger::LevelFilter::Error,
            Self::Info => logger::LevelFilter::Info,
            Self::Trace => logger::LevelFilter::Trace,
            Self::Warn => logger::LevelFilter::Warn,
        }
    }
}
