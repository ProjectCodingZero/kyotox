use std::collections::VecDeque;

use crate::cursor::Cursor;

pub mod cell;

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Row {
    cells: Vec<cell::Cell>,
}


#[derive(Debug, PartialEq, Eq, Default)]
pub struct Grid {
    viewport: Vec<Row>,
    scrollback: VecDeque<Row>,
    cols: usize,
    rows: usize,
    max_scrollback: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinates{
    pub col: usize,
    pub row: usize,
}

impl From<(usize, usize)> for Coordinates {
    fn from((col, row): (usize, usize)) -> Self {
        Coordinates{col, row}
    }
}

impl From<Cursor> for Coordinates {
    fn from(cursor: Cursor) -> Self {
        (cursor.x as usize, cursor.y as usize).into()
    }
}

impl From<&Cursor> for Coordinates {
    fn from(cursor: &Cursor) -> Self {
        (*cursor).into()
    }
}
impl Row {
    pub fn new(rows: usize) -> Self {
        Self {
            cells: vec![cell::Cell::default(); rows],
        }
    }
    pub fn resize(&mut self, cols: usize) {
        self.cells.resize(cols, cell::Cell::default());
    }
}
impl Grid {
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        let viewport = vec![Row::new(rows); cols];
        Grid{
            viewport,
            scrollback: VecDeque::new(),
            cols,
            rows,
            max_scrollback,
        }
    }
    pub fn resize(&mut self, rows: usize, cols: usize) {
        if rows == 0 || cols == 0 {
            return;
        }
        
        self.viewport.resize_with(rows, || Row {
            cells: vec![cell::Cell::default(); cols],
        });
        for row in &mut self.viewport {
            row.cells.resize(cols, cell::Cell::default());
        }
        self.rows = rows;
        self.cols = cols;
    }

    /// Sets the character at the cursor position.
    ///
    /// Returns `false` when the cursor is outside the grid.
    pub fn set_char<C>(&mut self, cursor: C, ch: char) -> bool
    where
        C: Into<Coordinates>,
    {
        let coordinates = cursor.into();
        let Some(row) = self.viewport.get_mut(coordinates.row) else {
            return false;
        };
        let Some(cell) = row.cells.get_mut(coordinates.col) else {
            return false;
        };

        cell.ch = ch;
        true
    }

    pub fn char_at<C>(&self, cursor: C) -> Option<char>
    where
        C: Into<Coordinates>,
    {
        let coordinates = cursor.into();
        self.viewport
            .get(coordinates.row)
            .and_then(|row| row.cells.get(coordinates.col))
            .map(|cell| cell.ch)
    }

    pub fn save_buffer(&mut self) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_sets_character_in_grid() {
        let mut grid = Grid::new(2, 3, 0);
        let cursor = Cursor::from((1, 0));

        assert!(grid.set_char(cursor, 'x'));
        assert_eq!(grid.char_at(cursor), Some('x'));
    }

    #[test]
    fn setting_character_outside_grid_is_rejected() {
        let mut grid = Grid::new(2, 3, 0);

        assert!(!grid.set_char(Cursor::from((3, 0)), 'x'));
        assert!(!grid.set_char(Cursor::from((0, 2)), 'x'));
    }

    #[test]
    fn resize_preserves_existing_characters() {
        let mut grid = Grid::new(1, 1, 0);
        grid.set_char(Cursor::default(), 'x');

        grid.resize(2, 2);

        assert_eq!(grid.char_at(Cursor::default()), Some('x'));
        assert_eq!(grid.char_at(Cursor::from((1, 1))), Some('\0'));
    }
}