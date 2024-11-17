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
    Escape,
    CursorPos(u32, u32),
    CursorUp(u32),
    CursorDown(u32),
    CursorForward(u32),
    CursorBackward(u32),
    CursorSave,
    CursorRestore,
    EraseDisplay(u8),
    EraseLine,
    SetGraphicsMode(Vec<u8, 5>),
    SetMode(u8),
    ResetMode(u8),
    HideCursor,
    ShowCursor,
    CursorToApp,
    SetNewLineMode,
    SetCol132,
    SetSmoothScroll,
    SetReverseVideo,
    SetOriginRelative,
    SetAutoWrap,
    SetAutoRepeat,
    SetInterlacing,
    SetLineFeedMode,
    SetCursorKeyToCursor,
    SetVT52,
    SetCol80,
    SetJumpScrolling,
    SetNormalVideo,
    SetOriginAbsolute,
    ResetAutoWrap,
    ResetAutoRepeat,
    ResetInterlacing,
    EnableMotionMouseTracking,
    DisableMotionMouseTracking,
    EnableFocusMode,
    DisableFocusMode,
    EnableSGRMouseMode,
    DisableSGRMouseMode,
    ShowAlternateBuffer,
    ShowNormalBuffer,
    EnableBracketedPasteMode,
    DisableBracketedPasteMode,
    SetTopAndBottom(u32, u32),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OSCSequence {
    ResetCursorColor,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ESCSequence {
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
            Escape => write!(formatter, "\u{1b}"),
            CursorPos(line, col) => write!(formatter, "{};{}H", line, col),
            CursorUp(amt) => write!(formatter, "{}A", amt),
            CursorDown(amt) => write!(formatter, "{}B", amt),
            CursorForward(amt) => write!(formatter, "{}C", amt),
            CursorBackward(amt) => write!(formatter, "{}D", amt),
            CursorSave => write!(formatter, "s"),
            CursorRestore => write!(formatter, "u"),
            EraseDisplay(n) => match n {
                0 => write!(formatter, "J"),
                1 => write!(formatter, "1J"),
                2 => write!(formatter, "2J"),
                _ => unreachable!(),
            },
            EraseLine => write!(formatter, "K"),
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
            ShowCursor => write!(formatter, "?25h"),
            HideCursor => write!(formatter, "?25l"),
            CursorToApp => write!(formatter, "?1h"),
            SetNewLineMode => write!(formatter, "20h"),
            SetCol132 => write!(formatter, "?3h"),
            SetSmoothScroll => write!(formatter, "?4h"),
            SetReverseVideo => write!(formatter, "?5h"),
            SetOriginRelative => write!(formatter, "?6h"),
            SetAutoWrap => write!(formatter, "?7h"),
            SetAutoRepeat => write!(formatter, "?8h"),
            SetInterlacing => write!(formatter, "?9h"),
            SetLineFeedMode => write!(formatter, "20l"),
            SetCursorKeyToCursor => write!(formatter, "?1l"),
            SetVT52 => write!(formatter, "?2l"),
            SetCol80 => write!(formatter, "?3l"),
            SetJumpScrolling => write!(formatter, "?4l"),
            SetNormalVideo => write!(formatter, "?5l"),
            SetOriginAbsolute => write!(formatter, "?6l"),
            ResetAutoWrap => write!(formatter, "?7l"),
            ResetAutoRepeat => write!(formatter, "?8l"),
            ResetInterlacing => write!(formatter, "?9l"),
            EnableMotionMouseTracking => write!(formatter, "?1002h"),
            DisableMotionMouseTracking => write!(formatter, "?1002l"),
            EnableFocusMode => write!(formatter, "?1004h"),
            DisableFocusMode => write!(formatter, "?1004l"),
            EnableSGRMouseMode => write!(formatter, "?1006h"),
            DisableSGRMouseMode => write!(formatter, "?1006l"),
            ShowAlternateBuffer => write!(formatter, "?1049h"),
            ShowNormalBuffer => write!(formatter, "?1049l"),
            EnableBracketedPasteMode => write!(formatter, "?2004h"),
            DisableBracketedPasteMode => write!(formatter, "?2004l"),
            SetTopAndBottom(x, y) => write!(formatter, "{};{}r", x, y),
        }
    }
}

impl Display for OSCSequence {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> DisplayResult {
        write!(formatter, "\u{1b}]")?;

        use OSCSequence::*;
        match self {
            ResetCursorColor => write!(formatter, "112\x07"),
        }
    }
}

impl Display for ESCSequence {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> DisplayResult {
        write!(formatter, "\u{1b}")?;

        use ESCSequence::*;
        match self {
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