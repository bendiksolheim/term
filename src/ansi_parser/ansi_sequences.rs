use core::fmt::{Display, Formatter, Result as DisplayResult};
use heapless::Vec;

#[derive(Debug, PartialEq, Clone)]
pub enum AnsiSequence {
    CSI(CSISequence),
    OSC(OSCSequence),
    ESC(ESCSequence),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CSISequence {
    CursorPos(u32, u32),
    CursorUp(u32),
    CursorDown(u32),
    CursorForward(u32),
    CursorBackward(u32),
    LinePositionAbsolute(u32),
    CursorCharacterAbsolute(u32),
    CursorStyle(u8),
    CursorSave,
    CursorRestore,
    DecPrivateModeSet(u32),
    DecPrivateModeReset(u32),
    EraseDisplay(u8),
    EraseCharacters(u32),
    EraseInLine(u32),
    SetGraphicsMode(Vec<u8, 5>),
    SetMode(u8),
    ResetMode(u8),
    SetNewLineMode,
    SetLineFeedMode,
    SetTopAndBottom(u32, u32),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OSCSequence {
    ResetCursorColor,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ESCSequence {
    Escape,
    SetAlternateKeypad,
    SetNumericKeypad,
    SetSingleShift2,
    SetSingleShift3,
    SetUKG0,
    SetUKG1,
    SetUSG0,
    SetUSG1,
    SetG0SpecialChars,
    SetG1SpecialChars,
    SetG0AlternateChar,
    SetG1AlternateChar,
    SetG0AltAndSpecialGraph,
    SetG1AltAndSpecialGraph,
    ReverseIndex,
}

impl Display for AnsiSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        use AnsiSequence::*;
        match self {
            CSI(csi_sequence) => csi_sequence.fmt(f),
            OSC(osc_sequence) => osc_sequence.fmt(f),
            ESC(esc_sequence) => esc_sequence.fmt(f),
        }
    }
}

impl Display for CSISequence {
    fn fmt(&self, formatter: &mut Formatter) -> DisplayResult {
        write!(formatter, "\u{1b}[")?;

        use CSISequence::*;
        match self {
            CursorPos(line, col) => write!(formatter, "{};{}H", line, col),
            CursorUp(amt) => write!(formatter, "{}A", amt),
            CursorDown(amt) => write!(formatter, "{}B", amt),
            CursorForward(amt) => write!(formatter, "{}C", amt),
            CursorBackward(amt) => write!(formatter, "{}D", amt),
            LinePositionAbsolute(n) => write!(formatter, "{}d", n),
            CursorCharacterAbsolute(n) => write!(formatter, "{}G", n),
            CursorStyle(s) => write!(formatter, "{} q", s),
            CursorSave => write!(formatter, "s"),
            CursorRestore => write!(formatter, "u"),
            DecPrivateModeSet(n) => write!(formatter, "?{}h", n),
            DecPrivateModeReset(n) => write!(formatter, "?{}l", n),
            EraseDisplay(n) => match n {
                0 => write!(formatter, "J"),
                1 => write!(formatter, "1J"),
                2 => write!(formatter, "2J"),
                _ => unreachable!(),
            },
            EraseInLine(n) => match n {
                0 => write!(formatter, "K"),
                _ => write!(formatter, "{}K", n),
            },
            EraseCharacters(n) => write!(formatter, "{}X", n),
            SetGraphicsMode(vec) => match vec.len() {
                0 => write!(formatter, "m"),
                1 => write!(formatter, "{}m", vec[0]),
                2 => write!(formatter, "{};{}m", vec[0], vec[1]),
                3 => write!(formatter, "{};{};{}m", vec[0], vec[1], vec[2]),
                5 => write!(formatter, "{};{};{};{};{}m", vec[0], vec[1], vec[2], vec[3], vec[4]),
                _ => unreachable!(),
            },
            SetMode(mode) => write!(formatter, "={}h", mode),
            ResetMode(mode) => write!(formatter, "={}l", mode),
            SetNewLineMode => write!(formatter, "20h"),
            SetLineFeedMode => write!(formatter, "20l"),
            SetTopAndBottom(x, y) => write!(formatter, "{};{}r", x, y),
        }
    }
}

impl Display for OSCSequence {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> DisplayResult {
        write!(formatter, "\u{1b}]")?;

        use OSCSequence::*;
        match self {
            ResetCursorColor => write!(formatter, "112\u{7}"),
        }
    }
}

impl Display for ESCSequence {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> DisplayResult {
        write!(formatter, "\u{1b}")?;

        use ESCSequence::*;
        match self {
            Escape => write!(formatter, "\u{1b}"),
            SetAlternateKeypad => write!(formatter, "="),
            SetNumericKeypad => write!(formatter, ">"),
            SetSingleShift2 => write!(formatter, "N"),
            SetSingleShift3 => write!(formatter, "O"),
            SetUKG0 => write!(formatter, "(A"),
            SetUKG1 => write!(formatter, ")A"),
            SetUSG0 => write!(formatter, "(B"),
            SetUSG1 => write!(formatter, ")B"),
            SetG0SpecialChars => write!(formatter, "(0"),
            SetG1SpecialChars => write!(formatter, ")0"),
            SetG0AlternateChar => write!(formatter, "(1"),
            SetG1AlternateChar => write!(formatter, ")1"),
            SetG0AltAndSpecialGraph => write!(formatter, "(2"),
            SetG1AltAndSpecialGraph => write!(formatter, ")2"),
            ReverseIndex => write!(formatter, "M"),
        }
    }
}

///This is what is outputted by the parsing iterator.
///Each block contains either straight-up text, or simply
///an ANSI escape sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum Output<'a> {
    TextBlock(&'a str),
    AnsiSequence(AnsiSequence),
}

impl<'a> Display for Output<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> DisplayResult {
        use Output::*;
        match self {
            TextBlock(txt) => write!(formatter, "{}", txt),
            AnsiSequence(seq) => write!(formatter, "{}", seq),
        }
    }
}
