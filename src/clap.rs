pub use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Command {
    #[arg(long, short='l', value_enum, default_value_t = LogLevel::Trace)]
    pub log: LogLevel,
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    #[default]
    Off,
}
