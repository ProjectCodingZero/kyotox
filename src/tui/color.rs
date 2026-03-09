use crossterm::{
    ExecutableCommand, event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub struct Color {
    Foreground: Color,
    Background: Color,
}
