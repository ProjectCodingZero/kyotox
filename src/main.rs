mod command;
mod tui;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::App;

fn main() -> color_eyre::Result<()> {
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

fn run_app(terminal: &mut ratatui::DefaultTerminal) -> color_eyre::Result<()> {
    let mut app = App::default();

    while !app.exit {
        app.handle_events()?;
        terminal.draw(|frame| {
            let area = frame.area();
            let main_block = Block::default()
                .borders(Borders::ALL)
                .title(app.title.trim());
            frame.render_widget(
                Paragraph::new(format!(
                    "{}{}",
                    app.text
                        .history
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<String>(),
                    app.text.actual.trim()
                ))
                .white()
                .on_black()
                .block(main_block),
                area,
            );
            frame.set_cursor_position(app.text.clone());
        })?;
    }
    Ok(())
}

/// Esta función asegura que si el programa "explota" (panic),
/// la terminal se limpie antes de mostrar el error.
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore(); // Recupera la terminal
        original_hook(panic_info); // Muestra el error original
    }));
}
