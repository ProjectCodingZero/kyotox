//! Core application loop, event handling, and TUI component rendering module.

use crate::{lua::ScriptEngineBridge, tui::SharedData};
use color_eyre::eyre::Result;
use crossterm::event::KeyModifiers;
use ratatui::{
    DefaultTerminal, Frame, buffer, layout::Position, style::Stylize, widgets::{Block, Borders, Paragraph},
};
use smol::{block_on, lock::RwLock};
use std::{fmt::Display, ops::Deref, path::Path, sync::Arc};

/// Point represents a specific 2D coordinate on the screen buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    /// Horizontal coordinate (column / X axis).
    pub x: u16,
    /// Vertical coordinate (row / Y axis).
    pub y: u16,
}

/// Represents the state of the text editor, including cursor position,
/// current buffer contents, and execution line history.
#[derive(Debug, Clone)]
pub struct Text {
    /// Current position of the cursor.
    pub cursor: Point,
    /// Current position of the buffer.
    pub buffer_idx: usize,
    /// Current editable text line buffer.
    pub buffer: Arc<String>,
    /// History log of previously submitted state snapshots.
    pub history: Vec<Arc<str>>,
}

/// The core application state and runtime engine context for the TUI.
#[derive(Debug)]
pub struct App {
    /// Thread-safe reference-counted pointer to global shared application data.
    state: Arc<RwLock<SharedData>>,
    /// Smol channel receiver for state synchronization events.
    rx: smol::channel::Receiver<SharedData>,
    /// Embedded Lua scripting instance.
    lua: mlua::Lua,
    /// Flag indicating whether the application main loop should terminate.
    pub exit: bool,
}

/// Direction parameter for character deletion operations.
#[derive(Debug)]
pub enum Direction {
    /// Deletes the character preceding the current cursor position.
    Previous,
    /// Deletes the character following the current cursor position.
    Next,
}

/// Represents the current operational mode of the editor interface.
#[derive(Debug, PartialEq, Default, Clone)]
pub enum EditorMode {
    /// Default mode: Inserts typed characters at the current cursor position.
    #[default]
    Insert,
    /// Overwrites characters directly at the current cursor position.
    Replace,
    /// Mode allowing execution of terminal-level commands.
    Command,
}

/// Initializes a new `App` instance with default settings.
///
/// - Creates an unbounded channel [`smol::channel::unbounded`] for [`crate::tui::SharedData`].
/// - Sets up the [`mlua::Lua`] engine with thread-safe shared state [`std::sync::Arc`], [`smol::lock::RwLock`] and [`crate::tui::SharedData`].
/// - Binds the [`crate::lua::ScriptEngineBridge`] directly to the Lua global environment as `app`.
/// - Attempts to read and execute `init.lua`, logging warnings or errors if it fails.
impl Default for App {
    fn default() -> Self {
        let (tx, rx) = smol::channel::unbounded::<SharedData>();
        let lua = mlua::Lua::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let bridge = ScriptEngineBridge {
            tx,
            state: shared_data.clone(),
        };
        lua.globals().set("app", bridge).unwrap();
        let config_lua = "init.lua";
        match std::fs::read_to_string(Path::new(config_lua)) {
            Ok(script) => {
                let result = lua.load(&script).exec();
                if let Err(err) = result {
                    log::error!("{:#?}", err);
                }
            }
            Err(err) => {
                log::warn!("Lua error: {:#?}", err);
            }
        }
        Self {
            rx,
            lua,
            state: shared_data,
            exit: false,
        }
    }
}

impl App {
    /// Runs the application loop drawing frames and processing events while `app.exit` is false.
    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Flushes the current buffer to history and advances the line cursor.
    #[inline]
    fn save_text(&mut self) {
        block_on(async {
            let mut state = self.state.write().await;
            state.text.save_buffer();
        });
    }

    /// Renders the TUI frame widgets, invokes Lua `update()` callbacks, and positions the cursor.
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let update: mlua::Function = self.lua.globals().get("update").unwrap();
        update.call(()).unwrap_or(());
        let state = block_on(async { self.state.read().await });
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(state.title.trim());
        let history: Vec<_> = state
            .text
            .history
            .deref()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let buffer = state.text.buffer.as_ref();
        let mode = state.mode.to_string();
        let padding = buffer.len() as u16;

        frame.render_widget(
            Paragraph::new(format!("{}\n{}{}", history.join("\n"), mode, buffer))
                .green()
                .on_black()
                .block(main_block),
            area,
        );
        drop(state);
        let state = block_on(async { self.state.write().await });
        let position = Position {
            x: state.text.cursor.x + 2,
            y: state.text.cursor.y + 1,
        };
        frame.set_cursor_position(position);
    }

    /// Polls Crossterm terminal input events using the configured FPS interval.
    pub fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{self, Event};
        let state = block_on(async { self.state.read().await });
        let fps = state.fps;
        drop(state);
        if event::poll(std::time::Duration::from_millis(fps))? {
            match event::read()? {
                Event::Key(event) => self.event_key_press(event),
                Event::Mouse(_event) => {}
                Event::Resize(width, height) => {
                    log::trace!("{:#?} {:#?}", width, height)
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Processes keyboard modifier events (such as Ctrl keys for toggling modes).
    fn event_key_modifiers(&mut self, key_modifier: KeyModifiers) {
        let mut state = block_on(async { self.state.write().await });
        match key_modifier {
            KeyModifiers::CONTROL => {
                state.mode = match state.mode {
                    EditorMode::Insert => EditorMode::Command,
                    EditorMode::Command => EditorMode::Insert,
                    EditorMode::Replace => EditorMode::Insert,
                }
            }
            _ => {}
        }
    }

    /// Processes keyboard press events (character input, backspace, enter, arrow keys).
    fn event_key_press(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        if key.is_release() {
            return;
        }

        self.event_key_modifiers(key.modifiers);
        let mut state = block_on(async { self.state.write().await });
        match key.code {
            KeyCode::Char(ch) => {
                state.text.insert(ch);
            }
            KeyCode::Left => state.text.move_cursor(Direction::Previous),
            KeyCode::Right => state.text.move_cursor(Direction::Next),
            KeyCode::Enter => {
                if state.text.buffer.trim() == "exit" {
                    self.exit = true;
                } else {
                    drop(state);
                    self.save_text();
                }
            }
            KeyCode::Backspace => state.text.remove(Direction::Previous),
            KeyCode::Delete => state.text.remove(Direction::Next),

            KeyCode::Home => {
                state.text.cursor.x = 0;
                state.text.buffer_idx = 0;
            }
            KeyCode::End => state.text.cursor.x = state.text.buffer.len() as u16,
            _ => {}
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            buffer_idx: 0,
            buffer: Arc::new(String::new()),
            history: Vec::new(),
            cursor: Point::default(),
        }
    }
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorMode::Insert | EditorMode::Replace => write!(f, "$"),
            EditorMode::Command => write!(f, ">"),
        }
    }
}

impl Text {
    /// Saves the current buffer into history and resets the buffer for a new line.
    fn save_buffer(&mut self) {
        self.add_history();
        self.buffer = Arc::new(String::new());

        self.cursor = Point {
            x: 0,
            y: self.cursor.y + 1,
        };
    }
    /// Moves cursor and buffer index.
    pub fn move_cursor(&mut self, direction: Direction) {
        // Get the buffer
        let buffer = &self.buffer;

        match direction {
            Direction::Previous => {
                if let Some(prev_ch) =
                    buffer[..self.buffer_idx].chars().next_back()
                {
                    self.cursor.x = self.cursor.x.saturating_sub(1);
                    self.buffer_idx -= prev_ch.len_utf8();
                } else {
                    log::trace!("Cursor is on the start of the line");
                }
            }
            Direction::Next => {
                if let Some(next_ch) = buffer[self.buffer_idx..].chars().next() {
                    self.cursor.x = self.cursor.x.saturating_add(1);
                    self.buffer_idx += next_ch.len_utf8();
                } else {
                    log::trace!("Cursor is on the end of the line");
                }
            }
        }
        log::trace!("Cursor moved to {:#?}", self.cursor);
        log::trace!("Buffer index changed to {:#?}", self.buffer_idx);
    }

    /// Appends the current buffer content into the history list.
    fn add_history(&mut self) {
        let arc_str: Arc<str> = Arc::from(self.buffer.as_str());
        self.history.push(arc_str);
        self.buffer_idx = 0;
    }

    /// Appends a character to the text buffer.
    pub fn push(&mut self, ch: char) {
        let buf: &mut String = Arc::make_mut(&mut self.buffer);
        log::debug!("{:?}", ch);
        buf.push(ch);
    }

    /// Removes the last character from the text buffer.
    pub fn pop(&mut self) {
        let buf: &mut String = Arc::make_mut(&mut self.buffer);
        match buf.pop() {
            Some(ch) => self.buffer_idx -= ch.to_string().len(),
            None => log::debug!("Nothing to pop"),
        }
    }

    /// Inserts a character at the current cursor position and advances the cursor.
    pub fn insert(&mut self, ch: char) {
        let buf: &mut String = Arc::make_mut(&mut self.buffer);
        let cursor: &mut Point = &mut self.cursor;
        log::debug!("character: {:?}, Cursor: {:?}", ch, cursor);
        let str = ch.to_string();
        buf.insert_str(self.buffer_idx, str.as_str());
        self.buffer_idx += str.len();
        self.cursor.x += 1;
    }

    /// Removes a character in the specified direction relative to the cursor position.
    pub fn remove(&mut self, direction: Direction) {
        let buf: &mut String = Arc::make_mut(&mut self.buffer);
        let x = self.cursor.x as usize;
        match direction {
            Direction::Previous => {
                if x > 0 {
                    buf.remove(self.buffer_idx);
                    self.buffer_idx -= 1;
                    self.cursor.x = self.cursor.x.saturating_sub(1);
                }
            }
            Direction::Next => {
                if x < buf.len() {
                    buf.remove(self.buffer_idx);
                }
            }
        }
    }
    pub fn set_buffer(&mut self, text: String) {
        let len = text.len();
        self.buffer = Arc::from(text);
        self.buffer_idx += len;
        self.cursor.x += len as u16;
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Position {
            x: point.x,
            y: point.y,
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_utf8(){
        let string = String::from("ȓȒȑȜ");
        let mut text = Text::default();
        text.set_buffer(string.clone());
        let string_get = text.buffer.clone();
        assert_eq!(*string_get, string);
        assert_eq!(text.buffer_idx, string.len());
    }
}