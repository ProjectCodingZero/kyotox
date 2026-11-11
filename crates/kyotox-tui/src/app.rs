use std::io;

use ratatui::DefaultTerminal;
use ratatui::prelude::*;

use kyotox_core::grid::{Cell as TermCell, CellFlags, Color as KColor, UnderlineStyle};
use kyotox_core::{Config, term};

pub struct App {
    term: term::Terminal,
    exit: bool,
}

impl App {
    pub fn new(config: Config, height: u16, width: u16) -> Self {
        Self {
            term: term::Terminal::new(height, width, config),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.term.handle_event();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let grid = self.term.get_grid();
        let buffer = frame.buffer_mut();

        for (y, row) in grid
            .viewport()
            .iter()
            .take(area.height as usize)
            .enumerate()
        {
            for (x, cell) in row.cells().iter().take(area.width as usize).enumerate() {
                let position = (area.x + x as u16, area.y + y as u16);
                if let Some(target) = buffer.cell_mut(position) {
                    target.set_char(if cell.ch == '\0' { ' ' } else { cell.ch });
                    target.set_style(cell_style(cell));
                }
            }
        }

        let cursor = self.term.get_cursor();
        let x = area
            .x
            .saturating_add(cursor.x)
            .min(area.right().saturating_sub(1));
        let y = area
            .y
            .saturating_add(cursor.y)
            .min(area.bottom().saturating_sub(1));
        frame.set_cursor_position((x, y));
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        use crossterm::event::{self, Event, KeyCode, KeyModifiers};

        match event::read()? {
            Event::Key(key_event) => {
                let control_char = if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match key_event.code {
                        KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                            Some((c.to_ascii_lowercase() as u8) - b'a' + 1)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                if let Some(ctrl_byte) = control_char {
                    self.term.write_input(&[ctrl_byte])?;
                } else {
                    let mut bytes = Vec::new();
                    if key_event.modifiers.contains(KeyModifiers::ALT) {
                        bytes.push(0x1b);
                    }
                    match key_event.code {
                        KeyCode::Char(c) => {
                            let mut buffer = [0; 4];
                            bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                        }
                        KeyCode::Enter => bytes.push(b'\r'),
                        KeyCode::Backspace => bytes.push(0x7f),
                        KeyCode::Tab => bytes.push(b'\t'),
                        KeyCode::BackTab => bytes.extend_from_slice(b"\x1b[Z"),
                        KeyCode::Esc => bytes.push(0x1b),
                        KeyCode::Up => bytes.extend_from_slice(b"\x1b[A"),
                        KeyCode::Down => bytes.extend_from_slice(b"\x1b[B"),
                        KeyCode::Right => bytes.extend_from_slice(b"\x1b[C"),
                        KeyCode::Left => bytes.extend_from_slice(b"\x1b[D"),
                        KeyCode::Home => bytes.extend_from_slice(b"\x1b[H"),
                        KeyCode::End => bytes.extend_from_slice(b"\x1b[F"),
                        KeyCode::PageUp => bytes.extend_from_slice(b"\x1b[5~"),
                        KeyCode::PageDown => bytes.extend_from_slice(b"\x1b[6~"),
                        KeyCode::Delete => bytes.extend_from_slice(b"\x1b[3~"),
                        KeyCode::Insert => bytes.extend_from_slice(b"\x1b[2~"),
                        KeyCode::F(n) => bytes.extend_from_slice(f_key_sequence(n).as_bytes()),
                        _ => {}
                    }
                    if !bytes.is_empty() {
                        self.term.write_input(&bytes)?;
                    }
                }
            }
            Event::Resize(cols, rows) => {
                self.term.resize(rows as usize, cols as usize);
            }
            _ => {}
        }

        Ok(())
    }
}

fn f_key_sequence(n: u8) -> String {
    match n {
        1..=4 => format!("\x1bO{}", (b'P' + n - 1) as char),
        5 => "\x1b[15~".to_owned(),
        6 => "\x1b[17~".to_owned(),
        7 => "\x1b[18~".to_owned(),
        8 => "\x1b[19~".to_owned(),
        9 => "\x1b[20~".to_owned(),
        10 => "\x1b[21~".to_owned(),
        11 => "\x1b[23~".to_owned(),
        12 => "\x1b[24~".to_owned(),
        _ => String::new(),
    }
}

fn cell_style(cell: &TermCell) -> Style {
    let mut style = Style::default();

    if let Some(fg) = cell.fg() {
        style = style.fg(core_color_to_color(fg));
    }
    if let Some(bg) = cell.bg() {
        style = style.bg(core_color_to_color(bg));
    }

    let attrs = cell.attrs();
    if attrs.contains(CellFlags::BOLD) {
        style = style.add_modifier(Modifier::BOLD);
    }
    if attrs.contains(CellFlags::DIM) {
        style = style.add_modifier(Modifier::DIM);
    }
    if attrs.contains(CellFlags::ITALIC) {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if attrs.contains(CellFlags::UNDERLINE)
        || attrs.contains(CellFlags::DOUBLE_UNDERLINE)
        || attrs.contains(CellFlags::CURLY_UNDERLINE)
        || attrs.contains(CellFlags::DOTTED_UNDERLINE)
        || attrs.contains(CellFlags::DASHED_UNDERLINE)
        || cell.underline() != UnderlineStyle::None
    {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if attrs.contains(CellFlags::BLINK) {
        style = style.add_modifier(Modifier::SLOW_BLINK);
    }
    if attrs.contains(CellFlags::REVERSE) {
        style = style.add_modifier(Modifier::REVERSED);
    }
    if attrs.contains(CellFlags::HIDDEN) {
        style = style.add_modifier(Modifier::HIDDEN);
    }
    if attrs.contains(CellFlags::STRIKETHROUGH) {
        style = style.add_modifier(Modifier::CROSSED_OUT);
    }

    style
}

fn core_color_to_color(color: KColor) -> Color {
    match color {
        KColor::RGB(r, g, b) => Color::Rgb(r, g, b),
        KColor::Indexed(index) => Color::Indexed(index),
        KColor::Ascii(code) => ascii_color(code),
    }
}

fn ascii_color(code: u8) -> Color {
    match code {
        30 | 40 => Color::Black,
        31 | 41 => Color::Red,
        32 | 42 => Color::Green,
        33 | 43 => Color::Yellow,
        34 | 44 => Color::Blue,
        35 | 45 => Color::Magenta,
        36 | 46 => Color::Cyan,
        37 | 47 => Color::Gray,
        90 | 100 => Color::DarkGray,
        91 | 101 => Color::LightRed,
        92 | 102 => Color::LightGreen,
        93 | 103 => Color::LightYellow,
        94 | 104 => Color::LightBlue,
        95 | 105 => Color::LightMagenta,
        96 | 106 => Color::LightCyan,
        97 | 107 => Color::White,
        _ => Color::White,
    }
}
