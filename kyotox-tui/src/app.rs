use std::arch::x86_64::__get_cpuid_max;
use std::fmt::Debug;
use std::io;
use ratatui::{DefaultTerminal, prelude::*};
use ratatui::symbols::Marker::Block;
use ratatui::widgets::Paragraph;
use kyotox_core::{term, CliConfig, Config};

pub struct App{
    writer: Box<dyn std::io::Write + Send >,
    term: term::Terminal,
    exit: bool
}

impl App{
    pub fn new(config: Config, height: u16, width: u16) -> Self{
        let mut term = term::Terminal::new(height, width, config);
        let writer = term.take_writer().unwrap();
        Self{
            term,
            exit: false,
            writer,
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()>{
        while !self.exit{
            self.term.handle_event();
            terminal.draw(|frame| self.draw(frame));
            self.handle_events()?
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame){
        let cels = self.term.get_grid();
        let area = frame.area();
        let cursor = self.term.get_cursor();
        let cursor = (cursor.x, cursor.y);
        frame.render_widget(Paragraph::new("Kyotox").green().on_black(), area);
        frame.set_cursor_position(cursor);
    }

    pub fn handle_events(&mut self) -> io::Result<()>{
        use crossterm::event::{self, Event, KeyCode, KeyModifiers};

        if let Event::Key(key_event) = event::read().unwrap() {
            match key_event.code {
                KeyCode::Char(c) => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        let ctrl_byte = match c {
                            'a'..='z' => (c as u8) - b'a' + 1,
                            'A'..='Z' => (c as u8) - b'A' + 1,
                            'c' => 3,  // Ctrl+C (SIGINT)
                            'd' => 4,  // Ctrl+D (EOF)
                            _ => 0,
                        };
                        if ctrl_byte != 0 {
                            let _ = self.writer.write_all(&[ctrl_byte]);
                        }
                    } else {
                        let mut buf = [0; 4];
                        let bytes = c.encode_utf8(&mut buf).as_bytes();
                        let _ = self.writer.write_all(bytes);
                    }
                }
                KeyCode::Enter => {
                    let _ = self.writer.write_all(b"\r");
                }
                KeyCode::Backspace => {
                    let _ = self.writer.write_all(&[0x7F]); // O b"\x08" según el entorno
                }
                KeyCode::Up => {
                    let _ = self.writer.write_all(b"\x1b[A");
                }
                KeyCode::Down => {
                    let _ = self.writer.write_all(b"\x1b[B");
                }
                KeyCode::Right => {
                    let _ = self.writer.write_all(b"\x1b[C");
                }
                KeyCode::Left => {
                    let _ = self.writer.write_all(b"\x1b[D");
                }
                KeyCode::Tab => {
                    let _ = self.writer.write_all(b"\t");
                }
                KeyCode::Esc => {
                    let _ = self.writer.write_all(b"\x1b");
                }
                _ => {}
            }

            // Forzar el envío de los bytes acumulados
            let _ = self.writer.flush();
        }
        Ok(())
    }
}