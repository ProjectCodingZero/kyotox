use std::io::Read;
use thiserror::Error;
use crate::{cursor::Cursor, grid::Grid, parser::VteBridge, pty::PtySession, parser::Parser, config, pty};
use crate::grid::Color;
/// Represents errors that can occur during terminal operations.

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    Pty(#[from] pty::PtyError),
}
/// Represents the current operational mode of the editor interface.
#[derive(Debug, PartialEq, Default, Clone)]
pub enum TerminalMode {
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
    pub grid: Grid,
    pub cursor: Cursor,
    pty: Option<PtySession>,
    event_rx: smol::channel::Receiver<EventTerminal>,
    pub mode: TerminalMode,
}
impl Terminal {
    pub fn new(height: u16, width: u16, config: config::Config) -> Self {
        let (tx, rx) = smol::channel::unbounded::<EventTerminal>();

        //TODO: Make this configurable
        //TODO: FIX UNWRAP
        let mut pty = pty::PtySession::new((height, width)).unwrap();
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
            grid: Grid::new(height as usize, width as usize, config.cli.scrollback_lines),
            cursor: Cursor::default(),
            event_rx: rx,
            mode: TerminalMode::default(),
            pty: Some(pty),
        }
    }
    pub fn handle_event(&mut self) {
        let cursor = self.cursor;
        while let Ok(event) = self.event_rx.try_recv(){
            log::debug!("Event: {:?}", event);
            match event{
                EventTerminal::PrintChar(c) => _ =self.grid.set_char(cursor, c),
                EventTerminal::NewLine => self.grid.scroll_down(),
                EventTerminal::None => {},
                _ => {},
            }
        }
    }
    pub fn get_event_rx(&self) -> &smol::channel::Receiver<EventTerminal> {
        &self.event_rx
    }
    pub fn take_writer(&mut self) -> Result<Box<dyn std::io::Write + Send>, crate::pty::PtyError> {
        self.pty.as_mut().unwrap().take_writer()
    }
    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }
    pub fn get_pty(&self) -> &PtySession {
        self.pty.as_ref().unwrap()
    }
    pub fn get_cursor(&self) -> Cursor {
        self.cursor
    }
    pub fn resize(&mut self, height: usize, width: usize) {
        self.grid.resize(height, width);
    }
}
#[derive(PartialEq, Debug, Clone, Copy)]
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
