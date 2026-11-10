//! PTY-backed terminal session.
//!
//! This crate is the only place that combines terminal emulation
//! (`kyotox-term`) with an actual process (`kyotox-pty`). It owns the reader
//! thread and forwards raw PTY output chunks to the screen; parsing itself stays
//! inside `kyotox-term`.

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use kyotox_config::Config;
use kyotox_pty::{PtyError, PtySession, PtySize};
use kyotox_term::{Cursor, Grid, Screen, TerminalMode};

/// A running terminal: a PTY process plus its parsed screen state.
pub struct Terminal {
    screen: Screen,
    pty: Option<PtySession>,
    writer: Option<Box<dyn Write + Send>>,
    reader_rx: Receiver<Vec<u8>>,
    /// Current frontend mode.
    pub mode: TerminalMode,
}

impl Terminal {
    /// Creates a terminal session with a default shell.
    ///
    /// If the PTY cannot be created, a headless screen is returned and the error
    /// is logged. Use [`Terminal::try_new`] when the error must be handled.
    pub fn new(height: u16, width: u16, config: Config) -> Self {
        let scrollback = config.cli.scrollback_lines;

        match Self::try_new(height, width, config) {
            Ok(terminal) => terminal,
            Err(error) => {
                log::error!("Failed to start PTY session: {error}");
                let (_sender, reader_rx) = mpsc::channel();
                Self {
                    screen: Screen::new(height as usize, width as usize, scrollback),
                    pty: None,
                    writer: None,
                    reader_rx,
                    mode: TerminalMode::default(),
                }
            }
        }
    }

    /// Creates a terminal session with a default shell.
    pub fn try_new(height: u16, width: u16, config: Config) -> Result<Self, PtyError> {
        let mut pty = PtySession::new((height, width))?;
        let mut reader = pty.clone_reader()?;
        let writer = pty.take_writer()?;
        let (sender, reader_rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("kyotox-pty-reader".to_owned())
            .spawn(move || {
                let mut buffer = [0u8; 8192];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(read) => {
                            if sender.send(buffer[..read].to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            log::debug!("PTY reader stopped: {error}");
                            break;
                        }
                    }
                }
            })
            .map_err(|error| PtyError::SpawnError(error.to_string()))?;

        Ok(Self {
            screen: Screen::new(
                height as usize,
                width as usize,
                config.cli.scrollback_lines,
            ),
            pty: Some(pty),
            writer: Some(writer),
            reader_rx,
            mode: TerminalMode::default(),
        })
    }

    /// Parses all PTY output that arrived since the last call.
    pub fn handle_event(&mut self) {
        loop {
            match self.reader_rx.try_recv() {
                Ok(chunk) => self.screen.advance(&chunk),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Takes ownership of the PTY writer.
    ///
    /// The writer can only be taken once. Prefer [`Terminal::write_input`] when
    /// the terminal should keep ownership.
    pub fn take_writer(&mut self) -> Result<Box<dyn Write + Send>, PtyError> {
        self.writer
            .take()
            .ok_or_else(|| PtyError::WriterError("PTY writer already taken".to_owned()))
    }

    /// Writes input bytes to the PTY.
    pub fn write_input(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self.writer.as_mut() {
            Some(writer) => writer.write_all(bytes),
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "PTY writer is unavailable",
            )),
        }
    }

    pub fn get_grid(&self) -> &Grid {
        self.screen.grid()
    }

    pub fn get_cursor(&self) -> Cursor {
        self.screen.cursor()
    }

    pub fn title(&self) -> &str {
        self.screen.title()
    }

    pub fn get_pty(&self) -> Option<&PtySession> {
        self.pty.as_ref()
    }

    pub fn resize(&mut self, height: usize, width: usize) {
        self.screen.resize(height, width);
        if let Some(pty) = self.pty.as_mut() {
            pty.resize(PtySize::from((height as u16, width as u16)));
        }
    }
}
