//! Herrumbre is a Rust-based Terminal User Interface (TUI) application
//! built with Ratatui, Crossterm, Smol async runtime, and MLua integration.

mod cl_parser;
/// Command management and plugin system definitions.
pub mod command;
/// Logger initialization and configuration module.
pub mod logger;
/// Lua scripting bridge and integration.
pub mod lua;
/// Terminal User Interface components, application loop, and configuration.
pub mod tui;

pub use cl_parser::{Command, Parser};

use crate::tui::AppConfig;

/// Internal global application settings wrapper.
#[derive(Debug)]
struct Settings {
    /// Configuration for the TUI application.
    pub app: AppConfig,
}
