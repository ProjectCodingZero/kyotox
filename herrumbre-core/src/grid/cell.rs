use bitflags::bitflags;

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
    pub fn set_underline(&mut self, underline: UnderlineStyle) {
        self.underline = underline;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn print_cell_size() {
        println!("CellFlags (u16): {}", std::mem::size_of::<CellFlags>());
        println!("Cell completo: {}", std::mem::size_of::<Cell>());
    }
}
