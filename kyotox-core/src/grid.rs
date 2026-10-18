use std::collections::VecDeque;

pub mod cell;

#[derive(Debug, PartialEq, Eq, Default)]
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
    col: usize,
    row: usize,
}

impl From<(usize, usize)> for Coordinates {
    fn from((col, row): (usize, usize)) -> Self {
        Coordinates{col, row}
    }
}

impl Grid {
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        let viewport = Vec::<Row>::with_capacity(rows);
        Grid{
            viewport,
            scrollback: VecDeque::new(),
            cols,
            rows,
            max_scrollback,
        }
    }
    pub fn resize(&mut self, rows: usize, cols: usize) {
        if rows > 0{
            self.cols = cols;
        }
        if cols > 0{
            self.rows = rows;
        }
    }
    pub fn save_buffer(&mut self) {

    }
}