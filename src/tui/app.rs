use crossterm::event::{KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::Position,
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
};
use std::{
    fmt,
    io::{Result, Write},
    sync::Arc,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}
#[derive(Debug, Clone)]
pub struct Text {
    cursor_position: Point,
    pub actual: Arc<String>,
    pub history: Vec<Arc<String>>,
}
#[derive(Debug)]
pub struct App {
    mode: EditorMode,
    pub title: String,
    pub text: Text,
    pub exit: bool,
}
//Direction for delete
#[derive(Debug)]
pub enum Direction {
    Previous,
    Next,
}
#[derive(Debug, PartialEq, Default)]
enum EditorMode {
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
impl Default for App {
    fn default() -> Self {
        Self {
            text: Text::new(),
            title: String::from("Herrumbre"),
            mode: EditorMode::default(),
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
    fn save_text(&mut self) {
        self.text.new_actual();
    }
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.trim());
        let padding = self.mode.to_string().len() as u16;
        frame.render_widget(
            Paragraph::new(format!(
                "{}{}{}",
                self.text
                    .history
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<String>(),
                self.mode,
                self.text.actual.trim(),
            ))
            .white()
            .on_black()
            .block(main_block),
            area,
        );
        let position = Position {
            x: self.text.cursor_position.x + padding + 1,
            y: self.text.cursor_position.y + 1,
        };
        frame.set_cursor_position(position);
    }

    pub fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{self, Event, KeyEventKind};

        if event::poll(std::time::Duration::from_millis(500))? {
            match event::read()? {
                Event::Key(event) => self.event_key_press(event),
                Event::Mouse(_event) => {}

                // Handle mouse events here
                Event::Resize(width, height) => {
                    println!("{:#?} {:#?}", width, height)
                    // Handle terminal resize here
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn change_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }
    fn event_key_modifiers(&mut self, key_modifier: crossterm::event::KeyModifiers) {
        match key_modifier {
            KeyModifiers::CONTROL => {
                if (self.mode == EditorMode::Command) {
                    self.change_mode(EditorMode::Insert);
                } else {
                    self.change_mode(EditorMode::Command);
                }
            }
            _ => {}
        }
    }
    fn event_key_press(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        self.event_key_modifiers(key.modifiers);
        match key.code {
            KeyCode::Char(ch) => {
                // Handle character input (example: update title or buffer)
                self.text.insert(ch);
            }
            KeyCode::Left => {
                // Move cursor left
                let cursor = self.text.cursor_position.clone();
                if (cursor.x as usize) > 0 {
                    self.text.cursor_position.x -= 1;
                }
            }
            KeyCode::Right => {
                // Move cursor right
                let cursor = self.text.cursor_position.clone();
                if (cursor.x as usize) < self.text.actual.len() {
                    self.text.cursor_position.x += 1;
                }
            }
            KeyCode::Enter => {
                // Example: clear title or buffer
                if self.text.actual.trim() == "exit" {
                    self.exit = true;
                } else {
                    self.save_text();
                }
            }
            KeyCode::Backspace => {
                // Remove last character from title or buffer
                self.text.remove(Direction::Previous);
            }
            KeyCode::Delete => {
                // Remove next character from title or buffer
                self.text.remove(Direction::Next);
            }
            KeyCode::Home => {
                // Move cursor to beginning of line
                self.text.cursor_position.x = 0;
            }
            KeyCode::End => {
                // Move cursor to end of line
                self.text.cursor_position.x = self.text.actual.len() as u16;
            }
            _ => {}
        }
    }
    fn get_text(&self) -> &str {
        &self.title
    }
}
//Implementation for text
impl Text {
    pub fn new() -> Self {
        Self {
            actual: Arc::new(String::new()),
            history: Vec::new(),
            cursor_position: Point::default(),
        }
    }
    fn new_actual(&mut self) {
        self.push('\n');
        self.add_history();
        self.actual = Arc::new(String::new());

        self.cursor_position = Point {
            x: 0,
            y: self.cursor_position.y + 1,
        };
    }
    fn add_history(&mut self) {
        self.history.push(self.actual.clone());
    }
    pub fn push(&mut self, ch: char) {
        let actual: &mut String = Arc::make_mut(&mut self.actual);
        actual.push(ch);
    }
    pub fn pop(&mut self) {
        let actual: &mut String = Arc::make_mut(&mut self.actual);
        actual.pop();
    }
    pub fn insert(&mut self, ch: char) {
        let actual: &mut String = Arc::make_mut(&mut self.actual);
        let cursor: &Point = &mut self.cursor_position;
        actual.insert(cursor.x as usize, ch);
        self.cursor_position.x += 1;
    }
    pub fn remove(&mut self, direction: Direction) {
        let actual: &mut String = Arc::make_mut(&mut self.actual);
        let cursor: &mut Point = &mut self.cursor_position;
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
impl Into<Position> for Point {
    fn into(self) -> Position {
        Position {
            x: self.x,
            y: self.y,
        }
    }
}
