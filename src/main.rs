mod command;
mod tui;
use std::fmt::Error;
use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use crate::tui::App;

fn main() -> color_eyre::Result<()> {
    // Check if threads are available
    // 1. Configurar manejo de errores y pánicos
    color_eyre::install()?;
    setup_panic_hook();
    // 2. Inicializar terminal (Modo raw + Pantalla alterna)
    let mut terminal = ratatui::init();

    // 3. Ejecutar aplicación
    let app_result = run_app(&mut terminal);

    // 4. Restaurar terminal SIEMPRE al final
    ratatui::restore();

    app_result?;
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
fn set_thread_pool() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads_available())
        .build_global()
        .unwrap()
}
fn run_app(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut app = App::default();
    let pool = rayon::ThreadPoolBuilder::default().build().unwrap();

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
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).ok();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore(); // Recupera la terminal
        execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
        original_hook(panic_info); // Muestra el error original
    }));
}
