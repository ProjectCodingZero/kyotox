mod clap;
pub mod command;
pub mod lua;
pub mod tui;
pub use clap::{Command, Parser};
pub mod logger;
use crate::tui::AppConfig;

#[derive(Debug)]
struct Settings {
    pub app: AppConfig,
}
