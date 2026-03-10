use ratatui::{DefaultTerminal, Frame, layout::Position};
use std::{
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
    pub title: String,
    pub text: Text,
    pub exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            text: Text::new(),
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
    fn save_text(&mut self) {
        self.text.new_actual();
    }
    fn draw(&self, frame: &mut Frame) {
        todo!()
    }

    pub fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{self, Event, KeyCode, KeyEventKind};

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
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
                        KeyCode::Esc | KeyCode::F(1) => {
                            // Exit application
                            self.exit = true;
                        }
                        KeyCode::Enter => {
                            // Example: clear title or buffer
                            self.save_text();
                        }
                        KeyCode::Backspace => {
                            // Remove last character from title or buffer
                            self.text.remove(true);
                        }
                        KeyCode::Delete => {
                            // Remove next character from title or buffer
                            self.text.remove(false);
                        }
                        _ => {}
                    }
                }
                Event::Mouse(_mouse_event) => {
                    // Handle mouse events here
                }
                Event::Resize(_width, _height) => {
                    // Handle terminal resize here
                }
                _ => {}
            }
        }
        Ok(())
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
    pub fn remove(&mut self, previous: bool) {
        let actual: &mut String = Arc::make_mut(&mut self.actual);
        let cursor: &mut Point = &mut self.cursor_position;
        if previous {
            cursor.x = cursor.x.saturating_sub(1);
        }
        if (cursor.x as usize) < actual.len() {
            actual.remove(cursor.x as usize);
        }
        /*
        if cursor.x > 0 {
            cursor.x = cursor.x.saturating_sub(1);
            actual.remove(cursor.x as usize);
        }
         */
    }
}
impl Into<Position> for Text {
    fn into(self) -> Position {
        Position {
            x: self.cursor_position.x + 1,
            y: self.cursor_position.y + 1,
        }
    }
}
