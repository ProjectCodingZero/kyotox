mod app;
pub use app::App;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{event::EnableMouseCapture, terminal::enable_raw_mode};

#[cfg(not(target_family = "wasm"))]
pub fn backend(stderr: std::io::Stderr) {
    CrosstermBackend::new(stderr);
}
