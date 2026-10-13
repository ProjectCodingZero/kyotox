use crate::{cursor::Cursor, grid::Grid, parser::Parser, pty::PtySession};

/// Represents the current operational mode of the editor interface.
#[derive(Debug, PartialEq, Default, Clone)]
enum TerminalMode {
    /// Default mode: Inserts typed characters at the current cursor position.
    #[default]
    Insert,
    /// Overwrites characters directly at the current cursor position.
    Replace,
    /// Mode allowing execution of terminal-level commands.
    Command,
}
pub struct Terminal {
    grid: Grid,
    cursor: Cursor,
    pty: Option<PtySession>,
    parser: Parser,
    mode: TerminalMode,
}

impl std::fmt::Display for TerminalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalMode::Insert | TerminalMode::Replace => write!(f, "$"),
            TerminalMode::Command => write!(f, ">"),
        }
    }
}
