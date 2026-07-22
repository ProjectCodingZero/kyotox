use crossterm::event::{KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::Position,
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
};
use std::{
    fmt,
    sync::Arc,
    path::Path
};
use color_eyre::eyre::Result;
use log::{debug, error, trace, warn};
use smol::{block_on, lock::RwLock};
use crate::{lua::ScriptEngineBridge, tui::SharedData};

/// [`Point`] represents a specific position on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    /// Horizontal coordinate (column/X axis)
    pub x: u16,
    /// Vertical coordinate (row/Y axis)
    pub y: u16,
}

/// [`Text`] is used for state of cursor and buffer
///
/// Cursor and Buffer management for user
/// History contains the previous saved states
#[derive(Debug, Clone)]
pub struct Text {
    /// Current cursor position
    cursor: Point,
    /// Current text editing by user
    pub buffer: Arc<String>,
    /// Buffer that was saved by user
    pub history: Vec<Arc<str>>,
}
#[derive(Debug)]
pub struct App {
    state: Arc<RwLock<SharedData>>,
    rx: smol::channel::Receiver<SharedData>,
    lua: mlua::Lua,
    pub exit: bool,
}
//Direction for delete
#[derive(Debug)]
pub enum Direction {
    Previous,
    Next,
}
#[derive(Debug, PartialEq, Default, Clone)]
pub enum EditorMode {
    #[default]
    Insert,
    Replace,
    Command,
}

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorMode::Insert => write!(f, "$"),
            EditorMode::Replace => write!(f, "%"),
            EditorMode::Command => write!(f, ">"),
        }
    }
}

/// Initializes a new [`App`] instance with default settings.
///
/// - Creates an unbounded channel (`smol::channel::unbounded`) for `SharedData`.
/// - Sets up the `mlua::Lua` engine with thread-safe shared state (`Arc<RwLock>`).
/// - Binds the `ScriptEngineBridge` directly to the Lua global environment as `"app"`.
/// - Attempts to read and execute `init.lua`, logging warnings or errors if it fails.
impl Default for App {
    fn default() -> Self {
        let (tx, rx) = smol::channel::unbounded::<SharedData>();
        let lua = mlua::Lua::new();
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        let bridge = ScriptEngineBridge{
            tx,
            state: shared_data.clone(),
        };
        lua.globals().set("app", bridge).unwrap();
        let config_lua = "init.lua";
        match std::fs::read_to_string(&Path::new(config_lua)){
            Ok(script ) => {

                let result = lua.load(&script).exec();
                if let Err(err) = result{
                    error!("{}", err);
                }
            }
            Err(err) => {
                warn!("Lua error: {}", err);
            }
        }
        Self{
            rx,
            lua,
            state: shared_data,
            exit: false,
        }
    }
}
impl App {
    /// Run the app while [`app.exit`] is false
    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {

        while !self.exit {

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    #[inline]
    fn save_text(&mut self) {
        block_on(async {
            let mut state = self.state.write().await;
            state.text.save_buffer();
        });
    }
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let update: mlua::Function = self.lua.globals().get("update").unwrap();
        update.call(()).unwrap_or(());
        let state = block_on(async {self.state.read().await});
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(state.title.trim());
        let padding = state.mode.to_string().len() as u16;
        frame.render_widget(
            Paragraph::new(format!(
                "{}{}{}",
                state.text
                    .history
                    .iter()
                    .map(|s| s.trim())
                    .collect::<String>(),
                state.mode,
                state.text.buffer.trim(),
            ))
            .green()
            .on_black()
            .block(main_block),
            area,
        );
        drop(state);
        let state = block_on(async{
            self.state.write().await
        });
        let position = Position {
            x: state.text.cursor.x + padding + 1,
            y: state.text.cursor.y + 1,
        };
        frame.set_cursor_position(position);
    }
    pub fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{self, Event};
        let state = block_on(async { self.state.read().await });
        let fps = state.fps;
        drop(state);
        if event::poll(std::time::Duration::from_millis(fps))? {
            match event::read()? {
                Event::Key(event) => self.event_key_press(event),
                Event::Mouse(_event) => {}

                // Handle mouse events here
                Event::Resize(width, height) => {
                    trace!("{:#?} {:#?}", width, height)
                    // Handle terminal resize here
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn event_key_modifiers(&mut self, key_modifier: KeyModifiers) {
        let mut state = block_on(async {self.state.write().await});
        match key_modifier {
            KeyModifiers::CONTROL => {
                if state.mode == EditorMode::Command {
                    state.mode = EditorMode::Insert;
                } else {
                    state.mode = EditorMode::Command;
                }
            }
            _ => {}
        }
    }


    fn event_key_press(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        if key.is_release(){
            return;
        }

        self.event_key_modifiers(key.modifiers);
        let mut state = block_on(async {self.state.write().await});
        match key.code {
            KeyCode::Char(ch) => {
                // Handle character input (example: update title or buffer)
                state.text.insert(ch);
            }
            KeyCode::Left => {
                // Move cursor left
                let cursor = state.text.cursor;
                if (cursor.x as usize) > 0 {
                    state.text.cursor.x -= 1;
                }
            },
            KeyCode::Right => {
                // Move cursor right
                let cursor = state.text.cursor;
                if (cursor.x as usize) < state.text.buffer.len() {
                    state.text.cursor.x += 1;
                }
            }
            KeyCode::Enter => {
                // Example: clear title or buffer
                if state.text.buffer.trim() == "exit" {
                    self.exit = true;
                } else {
                    drop(state);
                    self.save_text();
                }
            }
            KeyCode::Backspace => {
                // Remove last character from title or buffer
                state.text.remove(Direction::Previous);
            }
            KeyCode::Delete => {
                // Remove next character from title or buffer
                state.text.remove(Direction::Next);
            }
            KeyCode::Home => {
                // Move cursor to beginning of line
                state.text.cursor.x = 0;
            }
            KeyCode::End => {
                // Move cursor to end of line
                state.text.cursor.x = state.text.buffer.len() as u16;
            }
            _ => {}
        }

    }
}
//Implementation for text
impl Default for Text{
    fn default() -> Self {
        Self{
            buffer: Arc::new(String::new()),
            history: Vec::new(),
            cursor: Point::default(),
        }
    }
}
impl Text {
    fn save_buffer(&mut self) {
        self.push('\n');
        self.add_history();
        self.buffer = Arc::new(String::new());

        self.cursor = Point {
            x: 0,
            y: self.cursor.y + 1,
        };
    }
    fn add_history(&mut self) {
        let arc_str: Arc<str> = Arc::from(self.buffer.as_str());
        self.history.push(arc_str);
    }
    pub fn push(&mut self, ch: char) {
        let actual: &mut String = Arc::make_mut(&mut self.buffer);
        actual.push(ch);
    }
    pub fn pop(&mut self) {
        let actual: &mut String = Arc::make_mut(&mut self.buffer);
        actual.pop();
    }
    pub fn insert(&mut self, ch: char) {
        let actual: &mut String = Arc::make_mut(&mut self.buffer);
        let cursor: &Point = &mut self.cursor;
        actual.insert(cursor.x as usize, ch);
        self.cursor.x += 1;
    }
    pub fn remove(&mut self, direction: Direction) {
        let actual: &mut String = Arc::make_mut(&mut self.buffer);
        let cursor: &mut Point = &mut self.cursor;
        match direction {
            Direction::Previous => {
                if cursor.x > 0 {
                    cursor.x = cursor.x.saturating_sub(1);
                    actual.remove(cursor.x as usize);
                }
            }
            Direction::Next => {
                if (cursor.x as usize) < actual.len() {
                    actual.remove(cursor.x as usize);
                }
            }
        }
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
