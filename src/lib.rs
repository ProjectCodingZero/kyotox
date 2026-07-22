pub mod command;
pub mod tui;
pub mod lua;
mod clap;
pub use clap::{Command, Parser};
pub mod logger;
use tui::SharedData;
use crate::tui::AppConfig;

#[derive(Debug)]
struct Settings{
    pub app: AppConfig,
}



