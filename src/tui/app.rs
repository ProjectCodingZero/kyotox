use std::{
    collections::HashMap,
    io::{Result, Write},
};

use crossterm::event;
use ratatui::{DefaultTerminal, Frame, Terminal, backend::CrosstermBackend, prelude::Backend};

#[derive(Debug)]
pub struct App {
    pub cursor_index: usize,
    pub title: String,
    pub exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cursor_index: 0,
            title: String::from("Herrumbre"),
            exit: false,
        }
    }
}

impl App {
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        todo!()
    }

    fn handle_events(&mut self) -> Result<()> {
        // Manejar eventos (Teclado)
        /*
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
        */
        Ok(())
    }
}
