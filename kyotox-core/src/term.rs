use std::io::Read;
use portable_pty::{CommandBuilder, PtySize};
use crate::{cursor::Cursor, grid::Grid, parser::VteBridge, pty::PtySession, parser::Parser, config, cursor};
use crate::grid::cell::Color;

/// Represents the current operational mode of the editor interface.
#[derive(Debug, PartialEq, Default, Clone)]
enum TerminalMode {
    /// Default mode: Inserts typed characters at the current cursor position.
    #[default]
    Insert,
    /// Overwrites characters directly at the current cursor position.
    Replace,
    /// Mode allowing execution of terminal-level commands.
    Command,
}
#[derive(Debug)]
pub struct Terminal {
    grid: Grid,
    cursor: Cursor,
    pty: Option<PtySession>,
    event_rx: smol::channel::Receiver<EventTerminal>,
    mode: TerminalMode,
}
impl Terminal {
    pub fn new(height: usize, width: usize, config: config::Config) -> Self {
        let (tx, rx) = smol::channel::unbounded::<EventTerminal>();

        let command = CommandBuilder::new_default_prog();
        let pty_size = PtySize{
            rows: height as u16,
            cols: width as u16,
            pixel_width: 0,
            pixel_height: 0,
        };
        let mut pty = PtySession::spawn(command, pty_size).unwrap();
        let mut reader = pty.clone_reader().unwrap();
        std::thread::spawn(move || {
            let mut parser = Parser::new();
            let mut vte = VteBridge::new(tx);
            let mut buffer = [0u8; 4096];
            loop{
                match reader.read(&mut buffer){
                    Ok(0) => break,
                    Ok(n) => {
                        let data = &buffer[..n];
                        parser.advance(&mut vte, data);
                    },
                    Err(e) => {
                        eprintln!("Error reading from PTY: {}", e);
                        break;
                    }
                }
            }
        });
        Self {
            grid: Grid::new(height, width, config.cli.scrollback_lines),
            cursor: Cursor::default(),
            event_rx: rx,
            mode: TerminalMode::default(),
            pty: Some(pty),
        }
    }
    pub fn resize(&mut self, height: usize, width: usize) {
        self.grid.resize(height, width);
    }
}
#[derive(PartialEq, Debug)]
pub enum EventTerminal{
    PrintChar(char),
    NewLine,
    ChangeColor(Color),
    ClearScreen,
    None,
}

impl std::fmt::Display for TerminalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalMode::Insert | TerminalMode::Replace => write!(f, "$"),
            TerminalMode::Command => write!(f, ">"),
        }
    }
}
