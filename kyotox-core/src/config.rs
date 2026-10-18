use std::fs;
use serde::Deserialize;
use thiserror::Error;
use crate::logger::LevelFilter;

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
    debug: bool,
    cli: CliConfig,
}

#[derive(Debug, Deserialize)]
pub struct CliConfig {
    log: LevelFilter,
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(ConfigError::Io(e)),
    };
    let config: Config = toml::from_str(&content).unwrap_or_else(|e| {
        Config::default()
    });
    Ok(config)
}
impl Default for CliConfig {
    fn default() -> Self {
        Self{
            log: LevelFilter::Off,
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
}