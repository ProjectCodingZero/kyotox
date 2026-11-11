//! Terminal screen state and the `vte` performer that updates it.
//!
//! `Screen` is the only place where escape sequences are interpreted. It owns
//! the [`Grid`], the cursor and the current pen, and applies PTY output directly
//! to the grid. It has no knowledge of PTYs, processes, configuration or UI.

use vte::{Params, Perform};

use crate::{Cell, CellFlags, Color, Cursor, Grid, UnderlineStyle};

/// The style currently applied to printed characters.
#[derive(Debug, Clone, Copy)]
struct Pen {
    fg: Option<Color>,
    bg: Option<Color>,
    attrs: CellFlags,
    underline: UnderlineStyle,
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            attrs: CellFlags::empty(),
            underline: UnderlineStyle::None,
        }
    }
}

/// A parsed terminal screen.
///
/// Feed arbitrary PTY chunks to [`Screen::advance`]. The parser keeps its state
/// between calls, so escape sequences may be split across chunks.
pub struct Screen {
    parser: vte::Parser,
    grid: Grid,
    cursor: Cursor,
    pen: Pen,
    saved_cursor: Cursor,
    title: String,
}

impl Screen {
    /// Creates a screen with `rows` visible rows and `cols` columns.
    pub fn new(rows: usize, cols: usize, max_scrollback: usize) -> Self {
        Self {
            parser: vte::Parser::new(),
            grid: Grid::new(rows, cols, max_scrollback),
            cursor: Cursor::default(),
            pen: Pen::default(),
            saved_cursor: Cursor::default(),
            title: String::new(),
        }
    }

    /// Feeds raw bytes read from a PTY into the parser.
    pub fn advance(&mut self, bytes: &[u8]) {
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());
        {
            let mut performer = Performer { screen: self };
            parser.advance(&mut performer, bytes);
        }
        self.parser = parser;
    }

    /// Resizes the visible grid, keeping the cursor inside it.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.grid.resize(rows, cols);
        self.clamp_cursor();
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// Clears the screen and resets the cursor and pen.
    pub fn reset(&mut self) {
        for row in self.grid.viewport_mut() {
            for cell in row.cells_mut() {
                *cell = Cell::default();
            }
        }
        self.cursor = Cursor::default();
        self.saved_cursor = Cursor::default();
        self.pen = Pen::default();
    }

    fn print(&mut self, c: char) {
        if self.grid.cols() == 0 || self.grid.rows() == 0 {
            return;
        }

        if self.cursor.x as usize >= self.grid.cols() {
            self.cursor.x = 0;
            self.line_feed();
        }

        if let Some(cell) = self.grid.cell_mut(self.cursor) {
            cell.ch = c;
            cell.set_foreground(self.pen.fg);
            cell.set_background(self.pen.bg);
            cell.set_attrs_flags(self.pen.attrs);
            cell.set_underline(self.pen.underline);
        }

        self.cursor.x = self.cursor.x.saturating_add(1);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' | 0x0b | 0x0c => self.line_feed(),
            b'\r' => self.cursor.x = 0,
            0x08 => self.cursor.x = self.cursor.x.saturating_sub(1),
            b'\t' => {
                let next = (self.cursor.x / 8 + 1) * 8;
                self.cursor.x = next.min(self.grid.cols().saturating_sub(1) as u16);
            }
            _ => {}
        }
    }

    fn line_feed(&mut self) {
        if self.grid.rows() == 0 {
            return;
        }

        if self.cursor.y as usize + 1 >= self.grid.rows() {
            self.grid.scroll_up();
        } else {
            self.cursor.y += 1;
        }
    }

    fn reverse_line_feed(&mut self) {
        if self.grid.rows() == 0 {
            return;
        }

        if self.cursor.y == 0 {
            self.grid.scroll_down();
        } else {
            self.cursor.y -= 1;
        }
    }

    fn clamp_cursor(&mut self) {
        let max_x = self.grid.cols().saturating_sub(1) as u16;
        let max_y = self.grid.rows().saturating_sub(1) as u16;
        self.cursor.x = self.cursor.x.min(max_x);
        self.cursor.y = self.cursor.y.min(max_y);
    }

    fn set_cursor(&mut self, x: usize, y: usize) {
        let max_x = self.grid.cols().saturating_sub(1) as u16;
        let max_y = self.grid.rows().saturating_sub(1) as u16;
        self.cursor.x = (x as u16).min(max_x);
        self.cursor.y = (y as u16).min(max_y);
    }

    fn move_up(&mut self, amount: usize) {
        self.cursor.y = self.cursor.y.saturating_sub(amount as u16);
    }

    fn move_down(&mut self, amount: usize) {
        let max_y = self.grid.rows().saturating_sub(1) as u16;
        self.cursor.y = self.cursor.y.saturating_add(amount as u16).min(max_y);
    }

    fn move_left(&mut self, amount: usize) {
        self.cursor.x = self.cursor.x.saturating_sub(amount as u16);
    }

    fn move_right(&mut self, amount: usize) {
        let max_x = self.grid.cols().saturating_sub(1) as u16;
        self.cursor.x = self.cursor.x.saturating_add(amount as u16).min(max_x);
    }

    fn clear_row(&mut self, row: usize) {
        if let Some(line) = self.grid.viewport_mut().get_mut(row) {
            for cell in line.cells_mut() {
                *cell = Cell::default();
            }
        }
    }

    fn clear_line_from(&mut self, col: usize, row: usize) {
        if let Some(line) = self.grid.viewport_mut().get_mut(row) {
            for cell in line.cells_mut().iter_mut().skip(col) {
                *cell = Cell::default();
            }
        }
    }

    fn clear_line_to(&mut self, col: usize, row: usize) {
        if let Some(line) = self.grid.viewport_mut().get_mut(row) {
            for cell in line.cells_mut().iter_mut().take(col + 1) {
                *cell = Cell::default();
            }
        }
    }

    fn erase_display(&mut self, mode: usize) {
        match mode {
            0 => {
                self.clear_line_from(self.cursor.x as usize, self.cursor.y as usize);
                for row in (self.cursor.y as usize + 1)..self.grid.rows() {
                    self.clear_row(row);
                }
            }
            1 => {
                for row in 0..self.cursor.y as usize {
                    self.clear_row(row);
                }
                self.clear_line_to(self.cursor.x as usize, self.cursor.y as usize);
            }
            _ => {
                for row in 0..self.grid.rows() {
                    self.clear_row(row);
                }
                if mode == 3 {
                    self.grid.clear_scrollback();
                }
            }
        }
    }

    fn erase_line(&mut self, mode: usize) {
        match mode {
            0 => self.clear_line_from(self.cursor.x as usize, self.cursor.y as usize),
            1 => self.clear_line_to(self.cursor.x as usize, self.cursor.y as usize),
            _ => self.clear_row(self.cursor.y as usize),
        }
    }

    fn apply_sgr(&mut self, values: &[u16]) {
        if values.is_empty() {
            self.pen = Pen::default();
            return;
        }

        let mut index = 0;
        while index < values.len() {
            let value = values[index];
            index += 1;

            match value {
                0 => self.pen = Pen::default(),
                1 => self.pen.attrs |= CellFlags::BOLD,
                2 => self.pen.attrs |= CellFlags::DIM,
                3 => self.pen.attrs |= CellFlags::ITALIC,
                4 => self.pen.attrs |= CellFlags::UNDERLINE,
                5 => self.pen.attrs |= CellFlags::BLINK,
                7 => self.pen.attrs |= CellFlags::REVERSE,
                8 => self.pen.attrs |= CellFlags::HIDDEN,
                9 => self.pen.attrs |= CellFlags::STRIKETHROUGH,
                21 => {
                    self.pen.attrs |= CellFlags::DOUBLE_UNDERLINE;
                    self.pen.underline = UnderlineStyle::Double;
                }
                22 => self.pen.attrs.remove(CellFlags::BOLD | CellFlags::DIM),
                23 => self.pen.attrs.remove(CellFlags::ITALIC),
                24 => {
                    self.pen.attrs.remove(
                        CellFlags::UNDERLINE
                            | CellFlags::DOUBLE_UNDERLINE
                            | CellFlags::CURLY_UNDERLINE
                            | CellFlags::DOTTED_UNDERLINE
                            | CellFlags::DASHED_UNDERLINE,
                    );
                    self.pen.underline = UnderlineStyle::None;
                }
                25 => self.pen.attrs.remove(CellFlags::BLINK),
                27 => self.pen.attrs.remove(CellFlags::REVERSE),
                28 => self.pen.attrs.remove(CellFlags::HIDDEN),
                29 => self.pen.attrs.remove(CellFlags::STRIKETHROUGH),
                30..=37 => self.pen.fg = Some(Color::Ascii(value as u8)),
                38 => self.parse_extended_color(value, &values, &mut index, true),
                39 => self.pen.fg = None,
                40..=47 => self.pen.bg = Some(Color::Ascii(value as u8)),
                48 => self.parse_extended_color(value, &values, &mut index, false),
                49 => self.pen.bg = None,
                53 => self.pen.attrs |= CellFlags::OVERLINE,
                55 => self.pen.attrs.remove(CellFlags::OVERLINE),
                90..=97 => self.pen.fg = Some(Color::Ascii(value as u8)),
                100..=107 => self.pen.bg = Some(Color::Ascii(value as u8)),
                _ => {}
            }
        }
    }

    fn parse_extended_color(
        &mut self,
        _code: u16,
        values: &[u16],
        index: &mut usize,
        foreground: bool,
    ) {
        let Some(kind) = values.get(*index).copied() else {
            return;
        };
        *index += 1;

        let color = match kind {
            5 => {
                let Some(indexed) = values.get(*index).copied() else {
                    return;
                };
                *index += 1;
                Color::Indexed(indexed as u8)
            }
            2 => {
                let Some(r) = values.get(*index).copied() else {
                    return;
                };
                let Some(g) = values.get(*index + 1).copied() else {
                    return;
                };
                let Some(b) = values.get(*index + 2).copied() else {
                    return;
                };
                *index += 3;
                Color::RGB(r as u8, g as u8, b as u8)
            }
            _ => return,
        };

        if foreground {
            self.pen.fg = Some(color);
        } else {
            self.pen.bg = Some(color);
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], ignore: bool, action: char) {
        if ignore {
            return;
        }

        let values: Vec<u16> = params.iter().flatten().copied().collect();
        let first = values.first().copied().unwrap_or(0) as usize;
        let amount = if first == 0 { 1 } else { first };

        match action {
            'A' => self.move_up(amount),
            'B' => self.move_down(amount),
            'C' => self.move_right(amount),
            'D' => self.move_left(amount),
            'E' => {
                self.move_down(amount);
                self.cursor.x = 0;
            }
            'F' => {
                self.move_up(amount);
                self.cursor.x = 0;
            }
            'G' => {
                let col = first.saturating_sub(1);
                self.set_cursor(col, self.cursor.y as usize);
            }
            'H' | 'f' => {
                let row = values.first().copied().unwrap_or(1).max(1) as usize - 1;
                let col = values.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                self.set_cursor(col, row);
            }
            'd' => {
                let row = first.max(1) - 1;
                self.set_cursor(self.cursor.x as usize, row);
            }
            'J' => self.erase_display(first),
            'K' => self.erase_line(first),
            'm' => self.apply_sgr(&values),
            's' => self.saved_cursor = self.cursor,
            'u' => self.cursor = self.saved_cursor,
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], ignore: bool, byte: u8) {
        if ignore {
            return;
        }

        match byte {
            b'7' => self.saved_cursor = self.cursor,
            b'8' => self.cursor = self.saved_cursor,
            b'D' => self.line_feed(),
            b'E' => {
                self.cursor.x = 0;
                self.line_feed();
            }
            b'M' => self.reverse_line_feed(),
            b'c' => self.reset(),
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() < 2 {
            return;
        }

        if matches!(params[0], b"0" | b"2") {
            self.title = String::from_utf8_lossy(params[1]).into_owned();
        }
    }
}

/// `vte` performer that forwards every parsed action to [`Screen`].
struct Performer<'a> {
    screen: &'a mut Screen,
}

impl Perform for Performer<'_> {
    fn print(&mut self, c: char) {
        self.screen.print(c);
    }

    fn execute(&mut self, byte: u8) {
        self.screen.execute(byte);
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        self.screen
            .csi_dispatch(params, intermediates, ignore, action);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        self.screen.esc_dispatch(intermediates, ignore, byte);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        self.screen.osc_dispatch(params, bell_terminated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_text_and_advances_the_cursor() {
        let mut screen = Screen::new(2, 8, 0);
        screen.advance(b"hi");

        assert_eq!(screen.grid().char_at(Cursor::from((0, 0))), Some('h'));
        assert_eq!(screen.grid().char_at(Cursor::from((1, 0))), Some('i'));
        assert_eq!(screen.cursor(), Cursor::from((2, 0)));
    }

    #[test]
    fn carriage_return_and_line_feed_move_the_cursor() {
        let mut screen = Screen::new(2, 8, 0);
        screen.advance(b"hello\r\nworld");

        assert_eq!(screen.grid().char_at(Cursor::from((0, 1))), Some('w'));
        assert_eq!(screen.cursor(), Cursor::from((5, 1)));
    }

    #[test]
    fn scrolling_moves_the_top_row_to_scrollback() {
        let mut screen = Screen::new(2, 4, 4);
        screen.advance(b"one\r\ntwo\r\nsix");

        assert_eq!(screen.grid().scrollback().len(), 1);
        assert_eq!(screen.grid().char_at(Cursor::from((0, 0))), Some('t'));
        assert_eq!(screen.grid().char_at(Cursor::from((0, 1))), Some('s'));
    }

    #[test]
    fn parses_colors_and_attributes() {
        let mut screen = Screen::new(1, 8, 0);
        screen.advance(b"\x1b[1;31mhi");

        let cell = &screen.grid().viewport()[0].cells()[0];
        assert_eq!(cell.fg(), Some(Color::Ascii(31)));
        assert!(cell.attrs().contains(CellFlags::BOLD));
    }

    #[test]
    fn parses_title() {
        let mut screen = Screen::new(1, 8, 0);
        screen.advance(b"\x1b]2;kyotox\x07");

        assert_eq!(screen.title(), "kyotox");
    }

    #[test]
    fn erase_display_clears_the_screen() {
        let mut screen = Screen::new(2, 4, 0);
        screen.advance(b"text\x1b[2J");

        for row in screen.grid().viewport() {
            for cell in row.cells() {
                assert_eq!(cell.ch, '\0');
            }
        }
    }
}
