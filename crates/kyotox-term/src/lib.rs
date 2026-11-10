mod parser;

use std::collections::VecDeque;

use bitflags::bitflags;

pub use parser::Screen;
/// Cursor represents the position of the cursor on the screen buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    /// Horizontal coordinate (column / X axis).
    pub x: u16,
    /// Vertical coordinate (row / Y axis).
    pub y: u16,
}
impl From<(u16, u16)> for Cursor {
    fn from((x, y): (u16, u16)) -> Self {
        Cursor{
            x,
            y
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    RGB(u8, u8, u8),
    Ascii(u8),
    Indexed(u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    fg: Option<Color>,
    bg: Option<Color>,
    attrs: CellFlags,
    underline: UnderlineStyle,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnderlineStyle {
    #[default]
    None = 0,
    Single = 1,
    Double = 2,
    Curly = 3,
    Dotted = 4,
    Dashed = 5,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CellFlags: u32{
        const BOLD             = 1 << 0;
        const DIM              = 1 << 1;
        const ITALIC           = 1 << 2;
        const UNDERLINE        = 1 << 3;
        const BLINK            = 1 << 4;
        const REVERSE          = 1 << 5;
        const HIDDEN           = 1 << 6;
        const STRIKETHROUGH    = 1 << 7;
        const OVERLINE         = 1 << 8;
        const WIDE_CHAR        = 1 << 9;
        const WIDE_CHAR_SPACER = 1 << 10;
        const DOUBLE_UNDERLINE = 1 << 11;
        const CURLY_UNDERLINE  = 1 << 12;
        const DOTTED_UNDERLINE = 1 << 13;
        const DASHED_UNDERLINE = 1 << 14;
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: '\0',
            fg: None,
            bg: None,
            attrs: CellFlags::empty(),
            underline: UnderlineStyle::None,
        }
    }
}
impl Cell {
    pub fn new(ch: char, fg: Option<Color>, bg: Option<Color>) -> Self {
        Self{
            ch,
            fg,
            bg,
            attrs: CellFlags::empty(),
            underline: UnderlineStyle::None,
        }
    }
    pub fn fg(&self) -> Option<Color> {
        self.fg
    }
    pub fn bg(&self) -> Option<Color> {
        self.bg
    }
    pub fn attrs(&self) -> CellFlags {
        self.attrs
    }
    pub fn underline(&self) -> UnderlineStyle {
        self.underline
    }
    pub fn set_foreground(&mut self, fg: Option<Color>) {
        self.fg = fg;
    }
    pub fn set_background(&mut self, bg: Option<Color>) {
        self.bg = bg;
    }
    pub fn add_attrs(&mut self, attrs: Vec<CellFlags>) {
        for attr in attrs {
            self.attrs |= attr;
        }
    }
    pub fn set_attrs(&mut self, attrs: Vec<CellFlags>) {
        self.attrs = CellFlags::empty();
        self.add_attrs(attrs);
    }
    pub fn set_attrs_flags(&mut self, attrs: CellFlags) {
        self.attrs = attrs;
    }
    pub fn set_underline(&mut self, underline: UnderlineStyle) {
        self.underline = underline;
    }
}
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Row {
    cells: Vec<Cell>,
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
    pub fn new(row_size: usize) -> Self {
        Self {
            cells: vec![Cell::default(); row_size],
        }
    }
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }
    pub fn resize(&mut self, cols: usize) {
        self.cells.resize(cols, Cell::default());
    }
}
impl Grid {
    /// Moves the visible content up by one row, pushing the top row into
    /// scrollback and appending a blank row at the bottom.
    pub fn scroll_up(&mut self) {
        if self.viewport.is_empty() {
            return;
        }

        let top = self.viewport.remove(0);
        if self.max_scrollback > 0 {
            self.scrollback.push_back(top);
            while self.scrollback.len() > self.max_scrollback {
                self.scrollback.pop_front();
            }
        }
        self.viewport.push(Row::new(self.cols));
    }

    /// Moves the visible content down by one row, inserting a blank row at the
    /// top and dropping the bottom row.
    pub fn scroll_down(&mut self) {
        if self.viewport.is_empty() {
            return;
        }

        self.viewport.insert(0, Row::new(self.cols));
        self.viewport.truncate(self.rows);
    }

    /// Removes every row from the scrollback buffer.
    pub fn clear_scrollback(&mut self) {
        self.scrollback.clear();
    }

    /// Returns the scrollback buffer.
    pub fn scrollback(&self) -> &VecDeque<Row> {
        &self.scrollback
    }
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        let viewport = vec![Row::new(cols); rows];
        Grid{
            viewport,
            scrollback: VecDeque::new(),
            cols,
            rows,
            max_scrollback,
        }
    }
    pub fn viewport(&self) -> &[Row] {
        &self.viewport
    }
    pub fn viewport_mut(&mut self) -> &mut [Row] {
        &mut self.viewport
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn resize(&mut self, row_size: usize, col_size: usize) {
        if row_size == 0 || col_size == 0 {
            return;
        }

        self.viewport.resize_with(row_size, || Row::new(col_size));
        for row in &mut self.viewport {
            row.resize(col_size);
        }
        self.rows = row_size;
        self.cols = col_size;
    }

    /// Returns a mutable reference to a single cell.
    pub fn cell_mut<C>(&mut self, cursor: C) -> Option<&mut Cell>
    where
        C: Into<Coordinates>,
    {
        let coordinates = cursor.into();
        self.viewport
            .get_mut(coordinates.row)
            .and_then(|row| row.cells.get_mut(coordinates.col))
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
    #[test]
    fn print_cell_size() {
        println!("CellFlags (u16): {}", std::mem::size_of::<CellFlags>());
        println!("Cell completo: {}", std::mem::size_of::<Cell>());
    }
}