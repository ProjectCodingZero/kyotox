//! Entry point for the Herrumbre application executable.
//!
//! Initializes logging, panic hooks, thread pools, terminal state via Ratatui,
//! and runs the main async event loop.

use color_eyre::eyre::{Result, eyre};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use herrumbre::{Command, Parser, logger::setup_logger, tui::App};
use log::{error, trace, warn};
use std::io;

/// Global memory allocator using `mimalloc` for high performance.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Application entry point.
///
/// Parses command-line arguments, sets up logging, panic hooks, global Rayon thread pool,
/// initializes the Ratatui terminal UI, and runs the application main loop.
fn main() -> Result<()> {
    let args = Command::parse();
    _ = setup_logger(args.log);
    // Start configure
    color_eyre::install()?;
    setup_panic_hook();
    _ = set_thread_pool();
    // Init terminal
    let mut terminal = ratatui::init();

    // Run app
    smol::block_on(async {
        trace!("Application initialized.");
        let mut app = App::default();

        while !app.exit {
            match app.handle_events() {
                Ok(()) => (),
                Err(e) => {
                    warn!("{}", e)
                }
            }
            match terminal.draw(|frame| app.draw(frame)) {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e)
                }
            }
        }
    });

    // Clean up terminal state on exit
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    ratatui::restore();

    Ok(())
}

/// Calculates the recommended number of threads to use in the global thread pool.
#[inline]
fn threads_available() -> usize {
    #[cfg(feature = "default")]
    {
        let threads = rayon::current_num_threads();
        if threads == 1 { threads } else { threads - 1 }
    }
    #[cfg(not(feature = "default"))]
    {
        1
    }
}

/// Initializes the global Rayon thread pool with the calculated thread count.
fn set_thread_pool() -> Result<()> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(threads_available())
        .build_global()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Thread pool init failed, {}", e);
            Err(eyre!("Error on thread pool init"))
        }
    }
}

/// Helper function to asynchronously run the main TUI event loop until exit.
async fn _run_app(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut app = App::default();

    while !app.exit {
        app.handle_events()?;
        terminal.draw(|frame| app.draw(frame))?;
    }
    Ok(())
}

/// Configures a custom panic hook to ensure the terminal state is restored
/// (leaving alternate screen mode and disabling mouse capture) before displaying
/// panic error messages.
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    execute!(io::stdout(), EnterAlternateScreen).ok();
    trace!("Setup hook initialised.");
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{}", panic_info);
        ratatui::restore(); // Restore terminal state
        execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
        original_hook(panic_info); // Display original panic information
    }));
}
