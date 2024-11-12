use crate::term::{colors::TerminalColor, graphics::Graphics};

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

    pub fn modify(&mut self, attributes: &[u8]) {
        if attributes.is_empty() {
            self.parse_attribute(Graphics::Reset);
        } else {
            self.modify_recursive(attributes);
        }
    }

    fn modify_recursive(&mut self, attributes: &[u8]) {
        match attributes[..] {
            [] => {
                // do nothing, every attribute is consumed
            }

            [38, 2, r, g, b, ref rest @ ..] => {
                // parse 24 bit color, set as foreground
                self.parse_attribute(Graphics::SetForeground(TerminalColor::TwentyFourBit(r, g, b)));
                self.modify_recursive(rest);
            }

            [48, 2, r, g, b, ref rest @ ..] => {
                // parse 24 bit color, set as background
                self.parse_attribute(Graphics::SetBackground(TerminalColor::TwentyFourBit(r, g, b)));
                self.modify_recursive(rest);
            }

            [38, 5, n, ref rest @ ..] => {
                // parse 8 bit color, set as foreground
                self.parse_attribute(Graphics::SetForeground(TerminalColor::EightBit(n)));
                self.modify_recursive(rest);
            }

            [48, 5, n, ref rest @ ..] => {
                // parse 8 bit color, set as background
                self.parse_attribute(Graphics::SetBackground(TerminalColor::EightBit(n)));
                self.modify_recursive(rest);
            }

            [n, ref rest @ ..] => {
                self.parse_attribute(Graphics::parse_ansi(&n));
                self.modify_recursive(rest);
            }
        }
    }

    fn parse_attribute(&mut self, attr: Graphics) {
        match attr {
            Graphics::Reset => *self = Self::default(),
            Graphics::Bold => self.weight = FontWeight::Bold,
            Graphics::Dim => self.weight = FontWeight::Dim,
            Graphics::Italic => self.italic = true,
            Graphics::Underline => self.underline = true,
            Graphics::ReverseVideo => self.reversed = true,
            Graphics::Strikethrough => self.strikethrough = true,
            Graphics::SetFont(_font) => {}
            Graphics::NotUnderlined => self.underline = false,
            Graphics::NotReversed => self.reversed = false,
            Graphics::SetForeground(color) => self.foreground = color,
            Graphics::SetBackground(color) => self.background = color,
        }
    }
}
