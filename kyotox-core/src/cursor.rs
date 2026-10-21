use crate::grid::Coordinates;

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
