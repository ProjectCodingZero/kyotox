pub mod app;
pub use app::App;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{event::EnableMouseCapture, terminal::enable_raw_mode};
use crate::tui::app::{EditorMode, Text};

#[derive(Debug, Clone)]
pub struct SharedData {
    //App Modified
    pub text: Text,
    mode: EditorMode,
    pub title: String,
    //User
    pub fps: u64,
    pub loading: Option<u16>,
    pub last_error: Option<String>,
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
#[derive(Debug, Clone)]
pub struct AppConfig{
    pub frame_refresh: u64,
}
impl Default for AppConfig {
    fn default() -> AppConfig {
        AppConfig{
            frame_refresh: 60
        }
    }
}
#[cfg(not(target_family = "wasm"))]
pub fn backend(stderr: std::io::Stderr) {
    CrosstermBackend::new(stderr);
}
