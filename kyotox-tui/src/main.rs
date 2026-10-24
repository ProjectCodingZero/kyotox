//! Entry point for the Kyotox Terminal application executable.
//!
//! Initializes logging, panic hooks, thread pools, terminal state via Ratatui,
//! and runs the main async event loop.
//!

mod clap_cli;
mod app;

use kyotox_core::{ky_async, load_config, load_config_async, logger};
use kyotox_core::thread::{set_thread_pool};
//Extern uses
use std::io;
use clap::Parser;
use crossterm::{
    execute,
    event::DisableMouseCapture,
    terminal::{LeaveAlternateScreen, EnterAlternateScreen
}};
use color_eyre::eyre::Result;

use crate::app::App;
use crate::clap_cli::CommandCli;
/// Global memory allocator using [`mimalloc::MiMalloc`] for high performance.
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Application entry point.
///
/// Parses command-line arguments, sets up logging, panic hooks, global thread pool,
/// initializes the [`ratatui`] terminal UI, and runs the application main loop.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let args = CommandCli::parse();
    _ = logger::setup_logger(args.log);
    // Start configure
    color_eyre::install()?;
    setup_panic_hook();
    _ = set_thread_pool();
    // Init terminal
    let mut terminal = ratatui::init();

    // Run app
    ky_async::block_on(async {
        let config = load_config_async("config.toml").await.unwrap();
        log::trace!("Application initialized.");
        let area = terminal.size().unwrap();
        let mut app = App::new(config, area.height, area.width);
        let _ = app.run(&mut terminal);
    });

    // Clean up terminal state on exit
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    ratatui::restore();

    Ok(())
}

/// Configures a custom panic hook to ensure the terminal state is restored
/// (leaving alternate screen mode and disabling mouse capture) before displaying
/// panic error messages.
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    execute!(io::stdout(), EnterAlternateScreen).ok();
    log::trace!("Setup hook initialised.");
    std::panic::set_hook(Box::new(move |panic_info| {
        log::error!("{}", panic_info);
        ratatui::restore(); // Restore terminal state
        execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
        original_hook(panic_info); // Display original panic information
    }));
}