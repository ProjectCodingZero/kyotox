//! Terminal User Interface (TUI) main module and shared data structures.

/// Application core implementation, state management, and rendering logic.
pub mod app;
pub use app::App;

use crate::tui::app::{EditorMode, Text};
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{event::EnableMouseCapture, terminal::enable_raw_mode};
use thiserror::Error;

/// Thread-safe shared application data container.
///
/// Holds the editor state, current window title, refresh timing, loading state,
/// and error tracking accessible across the TUI event loop and Lua bridge.
#[derive(Debug, Clone)]
pub struct SharedData {
    /// Active editor text buffer and cursor state.
    pub text: Text,
    /// Active editor operational mode.
    mode: EditorMode,
    /// Window or panel title displayed in the TUI block border.
    pub title: String,
    /// Event polling interval in milliseconds.
    pub fps: u64,
    /// Loading progress indicator state (0-100 percentage).
    pub loading: Option<u16>,
    /// Last error message string encountered by the application.
    pub last_error: Option<AppError>,
}
#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("Error config: ({0})")]
    Config(String),

    #[error("Error file not found: ({0})")]
    FileNotFound(String),
}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            fps: 500,
            loading: None,
            text: Text::default(),
            mode: EditorMode::default(),
            last_error: None,
            title: String::from("Herrumbre"),
        }
    }
}

/// Global TUI application runtime configuration parameters.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Frame refresh rate interval in milliseconds.
    pub frame_refresh: u64,
}

impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig { frame_refresh: 60 }
    }
}

/// Constructs a non-WASM Crossterm backend targeting standard error.
#[cfg(not(target_family = "wasm"))]
pub fn backend(stderr: std::io::Stderr) {
    CrosstermBackend::new(stderr);
}
