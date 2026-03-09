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
    let mut string = String::new();
    let app = App::default();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(
                Paragraph::new(string.trim()).white().on_black().block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.title.trim()),
                ),
                area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // Salir con 'q' solo si no estamos escribiendo
                        KeyCode::Char('q') if string.is_empty() => break,
                        KeyCode::Char(ch) => string.push(ch),
                        KeyCode::Delete | KeyCode::Backspace => {
                            string.pop();
                        }
                        KeyCode::Enter => {
                            if string.trim() == "exit" {
                                break;
                            }
                            string.clear();
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
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
