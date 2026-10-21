pub mod ansi;
pub mod cursor;
pub mod grid;
pub mod logger;
pub mod parser;
pub mod pty;
pub mod term;
pub mod error;
mod config;
pub mod thread;
pub mod ky_async;

pub use config::{
    Config,
    CliConfig,
    load_config,
    load_config_async,
};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }


}
