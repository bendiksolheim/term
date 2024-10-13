use iced::Color;

#[derive(Debug, Clone, Copy)]
pub enum TerminalColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

impl Default for TerminalColor {
    fn default() -> Self {
        TerminalColor::Default
    }
}

/*
"brightBlack": "#5B6078",
  "brightRed": "#ED8796",
  "brightGreen": "#A6DA95",
  "brightYellow": "#EED49F",
  "brightBlue": "#8AADF4",
  "brightPurple": "#F5BDE6",
  "brightCyan": "#8BD5CA",
  "brightWhite": "#A5ADCB"
*/

impl TerminalColor {
    pub fn foreground_color(&self) -> Color {
        match self {
            TerminalColor::Black => Color::from_rgb(0.286, 0.302, 0.392),
            TerminalColor::Red => Color::from_rgb(0.929, 0.529, 0.588),
            TerminalColor::Green => Color::from_rgb(0.651, 0.855, 0.584),
            TerminalColor::Yellow => Color::from_rgb(0.933, 0.831, 0.624),
            TerminalColor::Blue => Color::from_rgb(0.541, 0.678, 0.957),
            TerminalColor::Magenta => Color::from_rgb(0.961, 0.741, 0.902),
            TerminalColor::Cyan => Color::from_rgb(0.545, 0.835, 0.792),
            TerminalColor::White => Color::from_rgb(0.722, 0.753, 0.878),
            TerminalColor::Default => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    pub fn background_color(&self) -> Color {
        match self {
            TerminalColor::Black => Color::from_rgb(0.286, 0.302, 0.392),
            TerminalColor::Red => Color::from_rgb(0.929, 0.529, 0.588),
            TerminalColor::Green => Color::from_rgb(0.651, 0.855, 0.584),
            TerminalColor::Yellow => Color::from_rgb(0.933, 0.831, 0.624),
            TerminalColor::Blue => Color::from_rgb(0.541, 0.678, 0.957),
            TerminalColor::Magenta => Color::from_rgb(0.961, 0.741, 0.902),
            TerminalColor::Cyan => Color::from_rgb(0.545, 0.835, 0.792),
            TerminalColor::White => Color::from_rgb(0.722, 0.753, 0.878),
            TerminalColor::Default => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
        }
    }
}
