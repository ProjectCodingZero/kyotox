//! Initializes the global application logger using `fern`.
//!
//! Configures log output formatting, timestamping, stdout/stderr chaining,
//! and log file generation inside the `logs/` directory.
//!
//! # Errors
//! Returns [`fern::InitError`] if logger initialization or log file creation fails.

use chrono::Local;
use fern;
pub use log::{debug, error, info, log_enabled, trace, warn, LevelFilter};
use std::time::SystemTime;
pub fn setup_logger<T>(level: T) -> Result<(), fern::InitError> 
    where T: Into<LevelFilter>
{
    let level = level.into();
    let output_file = format!("logs/{}.log", Local::now().format("%H:%M:%S %d-%m-%Y"));
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        // .chain(std::io::stdout())
        // .chain(std::io::stderr())
        .chain(fern::log_file(output_file)?)
        .apply()?;
    Ok(())
}
