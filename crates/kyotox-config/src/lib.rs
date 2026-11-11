//! Initializes the global application logger using `fern`.
//!
//! Configures log output formatting, timestamping, stdout/stderr chaining,
//! and log file generation inside the `logs/` directory.
//!
//! # Errors
//! Returns [`fern::InitError`] if logger initialization or log file creation fails.
use std::{fs, time::SystemTime};
use serde::Deserialize;
use smol::io::AsyncReadExt;
use thiserror::Error;
use chrono::Local;
pub use log::{debug, error, info, log_enabled, trace, warn, LevelFilter};
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
#[derive(Error, Debug)]
pub enum ThreadError {
    #[error("Failed to initialize thread pool: {0}")]
    InitError(#[from] rayon::ThreadPoolBuildError),
}

/// Calculates the recommended number of threads to use in the global thread pool.
#[inline]
fn threads_available() -> usize {
    let threads = rayon::current_num_threads();
    if threads == 1 { threads } else { threads - 1 }
}
/// Initializes the global [`rayon::ThreadPool`] thread pool with the calculated thread count.
pub fn set_thread_pool() -> Result<(), ThreadError> {
    let threads = threads_available();
    match rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(ThreadError::from(e))
        }
    }
}
#[derive(Error, Debug)]
pub enum ConfigError{
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    #[error("The TOML file has an invalid format or missing fields: {0}")]
    InvalidConfig(String),
    #[error("Required configuration field is missing: {0}")]
    MissingField(String),
}
#[derive(Debug, Default, Deserialize)]
pub struct Config{
    pub debug: bool,
    pub cli: CliConfig,
}

#[derive(Debug, Deserialize)]
pub struct CliConfig {
    pub log: LevelFilter,
    pub scrollback_lines: usize,
}
impl Config{
    fn new(content: impl ToString) -> Result<Self, ConfigError>{
        let content = content.to_string();
        toml::from_str(&content)
            .map_err(|error| ConfigError::InvalidConfig(error.to_string()))
    }
}
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(ConfigError::Io(e)),
    };
    Config::new(content)
}
pub async fn load_config_async(path: &str) -> Result<Config, ConfigError> {
    let mut file = match async_fs::File::open(path).await {
        Ok(content) => content,
        Err(e) => return Err(ConfigError::Io(e)),
    };
    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content).await {
        return Err(ConfigError::Io(e));
    }
    Config::new(content)
}
impl Default for CliConfig {
    fn default() -> Self {
        Self{
            log: LevelFilter::Off,
            scrollback_lines: 10000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_test_path() -> String {
        format!("{}/tests", env!("CARGO_MANIFEST_DIR"))
    }
    /// This
    #[test]
    fn test_load_config() {
        let toml_config = format!("{}/config.toml",get_test_path());
        let config = load_config(toml_config.as_str());
        // Verify if config was loading successfully
        assert!(config.is_ok());
    }
    #[test]
    fn test_load_config_async(){
        smol::block_on(async {

            let toml_config = format!("{}/config.toml",get_test_path());
            let config = load_config_async(toml_config.as_str()).await;
            assert!(config.is_ok());
        });
    }
}