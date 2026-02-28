mod app;
use crossterm::{
    ExecutableCommand,
    event::{self, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Terminal, prelude::CrosstermBackend, style::Stylize};
use std::io::stdout;

use crate::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut string = String::new();
    let mut terminal = ratatui::init();

    let app = App::default();

    // 2. BUCLE PRINCIPAL DE LA APLICACIÓN
    loop {
        // Dibujar la interfaz
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new(format!(
                    "¡Pantalla de Terminal Activa!\nPresiona 'q' para salir\n{}",
                    string
                ))
                .white()
                .on_black()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.title.clone()),
                ),
                area,
            );
        })?;

        // Manejar eventos (Teclado)
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(ch) => string.push(ch),
                        KeyCode::Delete | KeyCode::Backspace => _ = string.pop(),
                        KeyCode::Enter => _ = string = String::new(),
                        KeyCode::F(1) => break,
                        _ => todo!(),
                    }
                }
            }
        }
    }

    // 3. RESTAURAR LA TERMINAL (MUY IMPORTANTE)
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
