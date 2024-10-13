use crate::terminal::{colors::TerminalColor, graphics::Graphics};

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

#[derive(Debug, Clone)]
pub struct CellStyle {
    pub foreground: TerminalColor,
    pub background: TerminalColor,
    pub reversed: bool,
    pub weight: FontWeight,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

#[derive(Debug, Clone)]
pub enum FontWeight {
    Dim,
    Normal,
    Bold,
}

impl CellStyle {
    pub fn default() -> Self {
        Self {
            foreground: TerminalColor::Default,
            background: TerminalColor::Default,
            reversed: false,
            weight: FontWeight::Normal,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }

    pub fn foreground_color(self) -> TerminalColor {
        if self.reversed {
            self.background
        } else {
            self.foreground
        }
    }

    pub fn background_color(self) -> TerminalColor {
        if self.reversed {
            self.foreground
        } else {
            self.background
        }
    }

    pub fn modify(&mut self, attributes: Vec<u8>) {
        let sgr: Vec<Graphics> = attributes.iter().map(|a| Graphics::parse_ansi(a)).collect();
        for attr in sgr {
            match attr {
                Graphics::Reset => *self = Self::default(),
                Graphics::Bold => self.weight = FontWeight::Bold,
                Graphics::Dim => self.weight = FontWeight::Dim,
                Graphics::Italic => self.italic = true,
                Graphics::Underline => self.underline = true,
                Graphics::ReverseVideo => self.reversed = true,
                Graphics::Strikethrough => self.strikethrough = true,
                Graphics::NotUnderlined => self.underline = false,
                Graphics::NotReversed => self.reversed = false,
                Graphics::SetForeground(color) => self.foreground = color,
                Graphics::SetBackground(color) => self.background = color,
            }
        }
    }
}
