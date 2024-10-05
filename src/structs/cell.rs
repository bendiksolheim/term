use crate::terminal::colors;

#[derive(Debug, Clone)]
pub struct Cell {
    pub content: char,
    pub style: CellStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            content: ' ',
            style: CellStyle::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CellStyle {
    pub foreground: colors::TerminalColor,
}
