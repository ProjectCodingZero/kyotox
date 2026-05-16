use std::fmt::Error;
use std::io;
use color_eyre::eyre::{eyre, Result};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use herrumbre::tui::App;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<()> {
    // Check if threads are available
    // Start configure
    color_eyre::install()?;
    setup_panic_hook();
    _ = set_thread_pool();
    // Init terminal
    let mut terminal = ratatui::init();

    // Run app
    let result = smol::block_on(async {
        let task = run_app(&mut terminal);
        
        task.await;
    });

    // execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();

    execute!(io::stdout(), LeaveAlternateScreen).ok();
    ratatui::restore();

    Ok(())
}
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

/// Init Thread pool global
fn set_thread_pool() -> Result<()> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(threads_available())
        .build_global(){
        Ok(_) => Ok(()),
        Err(e) => Err(eyre!("Error on thread pool init"))   ,
    }
}
async fn run_app(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut app = App::default();

    while !app.exit {
        app.handle_events()?;
        terminal.draw(|frame| app.draw(frame))?;
    }
    Ok(())
}

/// Esta función asegura que si el programa "explota" (panic),
/// la terminal se limpie antes de mostrar el error.
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    // execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).ok();
    execute!(io::stdout(), EnterAlternateScreen).ok();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore(); // Recupera la terminal
        execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
        original_hook(panic_info); // Muestra el error original
    }));
}
