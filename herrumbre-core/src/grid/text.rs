use std::sync;

use crate::grid::point::Point;


/// Represents the state of the text editor, including cursor position,
/// current buffer contents, and execution line history.
#[derive(Debug, Clone)]
pub struct Text {
    /// Current position of the cursor.
    pub cursor: Point,
    /// Current position of the buffer.
    pub buffer_idx: usize,
    /// Current editable text line buffer.
    pub buffer: sync::Arc<String>,
    /// History log of previously submitted state snapshots.
    pub history: Vec<sync::Arc<str>>,
}

#[derive(Debug)]
pub enum Direction {
    /// Deletes the character preceding the current cursor position.
    Previous,
    /// Deletes the character following the current cursor position.
    Next,
}

impl Text {
    /// Saves the current buffer into history and resets the buffer for a new line.
    fn save_buffer(&mut self) {
        self.add_history();
        self.buffer = sync::Arc::new(String::new());

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
        let arc_str: sync::Arc<str> = sync::Arc::from(self.buffer.as_str());
        self.history.push(arc_str);
        self.buffer_idx = 0;
    }

    /// Appends a character to the text buffer.
    pub fn push(&mut self, ch: char) {
        let buf: &mut String = sync::Arc::make_mut(&mut self.buffer);
        log::debug!("{:?}", ch);
        buf.push(ch);
    }

    /// Removes the last character from the text buffer.
    pub fn pop(&mut self) {
        let buf: &mut String = sync::Arc::make_mut(&mut self.buffer);
        match buf.pop() {
            Some(ch) => self.buffer_idx -= ch.to_string().len(),
            None => log::debug!("Nothing to pop"),
        }
    }

    /// Inserts a character at the current cursor position and advances the cursor.
    pub fn insert(&mut self, ch: char) {
        let buf: &mut String = sync::Arc::make_mut(&mut self.buffer);
        let cursor: &mut Point = &mut self.cursor;
        log::debug!("character: {:?}, Cursor: {:?}", ch, cursor);
        let str = ch.to_string();
        buf.insert_str(self.buffer_idx, str.as_str());
        self.buffer_idx += str.len();
        self.cursor.x += 1;
    }

    /// Removes a character in the specified direction relative to the cursor position.
    pub fn remove(&mut self, direction: Direction) {
        let buf: &mut String = sync::Arc::make_mut(&mut self.buffer);
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
        self.buffer = sync::Arc::from(text);
        self.buffer_idx += len;
        self.cursor.x += len as u16;
    }
}
/*
impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Position {
            x: point.x,
            y: point.y,
        }
    }
}
 */