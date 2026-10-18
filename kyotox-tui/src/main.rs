//! Entry point for the Kyotox Terminal application executable.
//!
//! Initializes logging, panic hooks, thread pools, terminal state via Ratatui,
//! and runs the main async event loop.
//!

mod clap_cli;
mod app;

use clap::Parser;
use kyotox_core::logger;
use kyotox_core::thread::{set_thread_pool};
//Extern uses
use std::io;
use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use color_eyre::eyre::Result;
use smol;

use crate::app::App;
use crate::clap_cli::CommandCli;
/// Global memory allocator using `mimalloc` for high performance.
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Application entry point.
///
/// Parses command-line arguments, sets up logging, panic hooks, global Rayon thread pool,
/// initializes the Ratatui terminal UI, and runs the application main loop.
fn main() -> Result<()> {
    let args = CommandCli::parse();
    _ = logger::setup_logger(args.log);
    // Start configure
    color_eyre::install()?;
    setup_panic_hook();
    _ = set_thread_pool();
    // Init terminal
    let mut terminal = ratatui::init();

    // Run app
    smol::block_on(async {
        log::trace!("Application initialized.");
        let mut app = App::default();
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
