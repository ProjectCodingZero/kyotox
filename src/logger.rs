use std::time::SystemTime;
use chrono::Local;
use crate::clap::LogLevel;
pub fn setup_logger(level: LogLevel) -> Result<(), fern::InitError> {
    let level = log_level(level);
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
        .chain(std::io::stdout())
        .chain(std::io::stderr())
        .chain(fern::log_file(output_file)?)
        .apply()?;
    Ok(())
}
fn log_level(level: LogLevel) -> log::LevelFilter {
    match level {
        LogLevel::Error => log::LevelFilter::Error,
        LogLevel::Warn => log::LevelFilter::Warn,
        LogLevel::Info => log::LevelFilter::Info,
        LogLevel::Debug => log::LevelFilter::Debug,
        LogLevel::Trace => log::LevelFilter::Trace,
        LogLevel::Off => log::LevelFilter::Off,
    }
}
