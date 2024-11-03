use crate::term::colors::TerminalColor;
use crate::term::font::Font;

/// Order of values comes from attribute number:
/// https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
#[derive(Debug, Clone, Copy)]
pub enum Graphics {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    ReverseVideo,
    Strikethrough,
    SetFont(Font),
    NotUnderlined,
    NotReversed,
    SetForeground(TerminalColor),
    SetBackground(TerminalColor),
}

impl Graphics {
    pub fn parse_ansi(value: &u8) -> Graphics {
        match value {
            0 => Self::Reset,
            1 => Self::Bold,
            2 => Self::Dim,
            3 => Self::Italic,
            4 => Self::Underline,
            7 => Self::ReverseVideo,
            9 => Self::Strikethrough,
            10..=19 => Self::SetFont(Font::Monospace), // TODO: Research how this works
            24 => Self::NotUnderlined,
            27 => Self::NotReversed,

            // Regular foreground colors
            30 => Self::SetForeground(TerminalColor::Black),
            31 => Self::SetForeground(TerminalColor::Red),
            32 => Self::SetForeground(TerminalColor::Green),
            33 => Self::SetForeground(TerminalColor::Yellow),
            34 => Self::SetForeground(TerminalColor::Blue),
            35 => Self::SetForeground(TerminalColor::Magenta),
            36 => Self::SetForeground(TerminalColor::Cyan),
            37 => Self::SetForeground(TerminalColor::White),
            39 => Self::SetForeground(TerminalColor::Default),

            // Regular background colors
            40 => Self::SetBackground(TerminalColor::Black),
            41 => Self::SetBackground(TerminalColor::Red),
            42 => Self::SetBackground(TerminalColor::Green),
            43 => Self::SetBackground(TerminalColor::Yellow),
            44 => Self::SetBackground(TerminalColor::Blue),
            45 => Self::SetBackground(TerminalColor::Magenta),
            46 => Self::SetBackground(TerminalColor::Cyan),
            47 => Self::SetBackground(TerminalColor::White),
            49 => Self::SetBackground(TerminalColor::Default),

            // Bright foreground colors
            90 => Self::SetForeground(TerminalColor::Black),
            91 => Self::SetForeground(TerminalColor::Red),
            92 => Self::SetForeground(TerminalColor::Green),
            93 => Self::SetForeground(TerminalColor::Yellow),
            94 => Self::SetForeground(TerminalColor::Blue),
            95 => Self::SetForeground(TerminalColor::Magenta),
            96 => Self::SetForeground(TerminalColor::Cyan),
            97 => Self::SetForeground(TerminalColor::White),

            // Bright background colors
            100 => Self::SetBackground(TerminalColor::Black),
            101 => Self::SetBackground(TerminalColor::Red),
            102 => Self::SetBackground(TerminalColor::Green),
            103 => Self::SetBackground(TerminalColor::Yellow),
            104 => Self::SetBackground(TerminalColor::Blue),
            105 => Self::SetBackground(TerminalColor::Magenta),
            106 => Self::SetBackground(TerminalColor::Cyan),
            107 => Self::SetBackground(TerminalColor::White),

            _ => unreachable!("Unimplemented SGR value found: {:?}", value),
        }
    }
}
