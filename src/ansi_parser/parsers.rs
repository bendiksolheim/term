#[cfg(test)]
mod tests;

use crate::ansi_parser::AnsiSequence;

use heapless::Vec;
use winnow::ascii::{digit0, digit1};
use winnow::combinator::{alt, delimited, opt, preceded};
use winnow::error::InputError;
use winnow::token::literal;
use winnow::{IResult, PResult, Parser};

macro_rules! tag_parser {
    ($sig:ident, $tag:expr, $ret:expr) => {
        fn $sig<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
            literal($tag).value($ret).parse_next(input)
        }
    };
}

fn parse_u32<'s>(input: &mut &'s str) -> PResult<u32, InputError<&'s str>> {
    digit1.try_map(|s: &str| s.parse::<u32>()).parse_next(input)
}

fn parse_u8<'s>(input: &mut &'s str) -> PResult<u8, InputError<&'s str>> {
    digit1.try_map(|s: &str| s.parse::<u8>()).parse_next(input)
}

// TODO kind of ugly, would prefer to pass in the default so we could use it for
// all escapes with defaults (not just those that default to 1).
fn parse_def_cursor_int<'s>(input: &mut &'s str) -> PResult<u32, InputError<&'s str>> {
    digit0.map(|s: &str| s.parse::<u32>().unwrap_or(1)).parse_next(input)
}

fn cursor_pos<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    (
        "[",
        parse_def_cursor_int,
        opt(";"),
        parse_def_cursor_int,
        alt(("H", "f")),
    )
        .map(|(_, x, _, y, _)| AnsiSequence::CursorPos(x, y))
        .parse_next(input)
}

fn escape<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    "\u{1b}".value(AnsiSequence::Escape).parse_next(input)
}

fn cursor_up<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[", parse_def_cursor_int, "A")
        .map(|am: u32| AnsiSequence::CursorUp(am))
        .parse_next(input)
}

fn cursor_down<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[", parse_def_cursor_int, "B")
        .map(|am| AnsiSequence::CursorDown(am))
        .parse_next(input)
}

fn cursor_forward<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[", parse_def_cursor_int, "C")
        .map(|am| AnsiSequence::CursorForward(am))
        .parse_next(input)
}

fn cursor_backward<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[", parse_def_cursor_int, "D")
        .map(|am| AnsiSequence::CursorBackward(am))
        .parse_next(input)
}

fn graphics_mode1<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[", parse_u8, "m")
        .map(|val| {
            let mode = Vec::from_slice(&[val]).expect("Vec::from_slice should allocate sufficient size");
            AnsiSequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode2<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    ("[", parse_u8, ";", parse_u8, "m")
        .map(|(_, val1, _, val2, _)| {
            let mode = Vec::from_slice(&[val1, val2]).expect("Vec::from_slice should allocate sufficient size");
            AnsiSequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode3<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    ("[", parse_u8, ";", parse_u8, ";", parse_u8, "m")
        .map(|(_, val1, _, val2, _, val3, _)| {
            let mode = Vec::from_slice(&[val1, val2, val3]).expect("Vec::from_slice should allocate sufficient size");
            AnsiSequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode4<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    "[m".value(AnsiSequence::SetGraphicsMode(Vec::new())).parse_next(input)
}

fn graphics_mode5<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    (
        "[", parse_u8, ";", parse_u8, ";", parse_u8, ";", parse_u8, ";", parse_u8, "m",
    )
        .map(|(_, val1, _, val2, _, val3, _, val4, _, val5, _)| {
            let mode = Vec::from_slice(&[val1, val2, val3, val4, val5])
                .expect("Vec::from_slice should allocate sufficient size");
            AnsiSequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    alt((
        graphics_mode1,
        graphics_mode2,
        graphics_mode3,
        graphics_mode4,
        graphics_mode5,
    ))
    .parse_next(input)
}

fn set_mode<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[=", parse_u8, "h")
        .map(|val| AnsiSequence::SetMode(val))
        .parse_next(input)
}

fn reset_mode<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[=", parse_u8, "l")
        .map(|val| AnsiSequence::ResetMode(val))
        .parse_next(input)
}

fn set_top_and_bottom<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    ("[", parse_u32, ";", parse_u32, "r")
        .map(|(_, x, _, y, _)| AnsiSequence::SetTopAndBottom(x, y))
        .parse_next(input)
}

fn erase_display<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    delimited("[", digit0.map(|s: &str| s.parse::<u8>().unwrap_or(0)), "J")
        .map(|n| AnsiSequence::EraseDisplay(n))
        .parse_next(input)
}

tag_parser!(cursor_save, "[s", AnsiSequence::CursorSave);
tag_parser!(cursor_restore, "[u", AnsiSequence::CursorRestore);
tag_parser!(erase_line, "[K", AnsiSequence::EraseLine);
tag_parser!(hide_cursor, "[?25l", AnsiSequence::HideCursor);
tag_parser!(show_cursor, "[?25h", AnsiSequence::ShowCursor);
tag_parser!(cursor_to_app, "[?1h", AnsiSequence::CursorToApp);
tag_parser!(set_new_line_mode, "[20h", AnsiSequence::SetNewLineMode);
tag_parser!(set_col_132, "[?3h", AnsiSequence::SetCol132);
tag_parser!(set_smooth_scroll, "[?4h", AnsiSequence::SetSmoothScroll);
tag_parser!(set_reverse_video, "[?5h", AnsiSequence::SetReverseVideo);
tag_parser!(set_origin_rel, "[?6h", AnsiSequence::SetOriginRelative);
tag_parser!(set_auto_wrap, "[?7h", AnsiSequence::SetAutoWrap);
tag_parser!(set_auto_repeat, "[?8h", AnsiSequence::SetAutoRepeat);
tag_parser!(set_interlacing, "[?9h", AnsiSequence::SetInterlacing);
tag_parser!(set_linefeed, "[20l", AnsiSequence::SetLineFeedMode);
tag_parser!(set_cursorkey, "[?1l", AnsiSequence::SetCursorKeyToCursor);
tag_parser!(set_vt52, "[?2l", AnsiSequence::SetVT52);
tag_parser!(set_col80, "[?3l", AnsiSequence::SetCol80);
tag_parser!(set_jump_scroll, "[?4l", AnsiSequence::SetJumpScrolling);
tag_parser!(set_normal_video, "[?5l", AnsiSequence::SetNormalVideo);
tag_parser!(set_origin_abs, "[?6l", AnsiSequence::SetOriginAbsolute);
tag_parser!(reset_auto_wrap, "[?7l", AnsiSequence::ResetAutoWrap);
tag_parser!(reset_auto_repeat, "[?8l", AnsiSequence::ResetAutoRepeat);
tag_parser!(reset_interlacing, "[?9l", AnsiSequence::ResetInterlacing);
tag_parser!(
    enable_bracketed_paste_mode,
    "[?2004h",
    AnsiSequence::EnableBracketedPasteMode
);
tag_parser!(
    disable_bracketed_paste_mode,
    "[?2004l",
    AnsiSequence::DisableBracketedPasteMode
);

tag_parser!(set_alternate_keypad, "=", AnsiSequence::SetAlternateKeypad);
tag_parser!(set_numeric_keypad, ">", AnsiSequence::SetNumericKeypad);
tag_parser!(set_uk_g0, "(A", AnsiSequence::SetUKG0);
tag_parser!(set_uk_g1, ")A", AnsiSequence::SetUKG1);
tag_parser!(set_us_g0, "(B", AnsiSequence::SetUSG0);
tag_parser!(set_us_g1, ")B", AnsiSequence::SetUSG1);
tag_parser!(set_g0_special, "(0", AnsiSequence::SetG0SpecialChars);
tag_parser!(set_g1_special, ")0", AnsiSequence::SetG1SpecialChars);
tag_parser!(set_g0_alternate, "(1", AnsiSequence::SetG0AlternateChar);
tag_parser!(set_g1_alternate, ")1", AnsiSequence::SetG1AlternateChar);
tag_parser!(set_g0_graph, "(2", AnsiSequence::SetG0AltAndSpecialGraph);
tag_parser!(set_g1_graph, ")2", AnsiSequence::SetG1AltAndSpecialGraph);
tag_parser!(set_single_shift2, "N", AnsiSequence::SetSingleShift2);
tag_parser!(set_single_shift3, "O", AnsiSequence::SetSingleShift3);

fn combined<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    // `alt` only supports up to 21 parsers, and winnow doesn't seem to
    // have an alternative with higher variability.
    // So we simply nest them.
    alt((
        alt((
            escape,
            cursor_pos,
            cursor_up,
            cursor_down,
            cursor_forward,
            cursor_backward,
            cursor_save,
            cursor_restore,
            erase_display,
            erase_line,
            graphics_mode,
            set_mode,
            reset_mode,
            hide_cursor,
            show_cursor,
            cursor_to_app,
            set_new_line_mode,
            set_col_132,
            set_smooth_scroll,
            set_reverse_video,
            set_origin_rel,
        )),
        alt((
            set_auto_wrap,
            set_auto_repeat,
            set_interlacing,
            set_linefeed,
            set_cursorkey,
            set_vt52,
            set_col80,
            set_jump_scroll,
            set_normal_video,
            set_origin_abs,
            reset_auto_wrap,
            reset_auto_repeat,
            reset_interlacing,
            enable_bracketed_paste_mode,
            disable_bracketed_paste_mode,
            set_top_and_bottom,
            set_alternate_keypad,
            set_numeric_keypad,
            set_uk_g0,
            set_uk_g1,
            set_us_g0,
        )),
        alt((
            set_us_g1,
            set_g0_special,
            set_g1_special,
            set_g0_alternate,
            set_g1_alternate,
            set_g0_graph,
            set_g1_graph,
            set_single_shift2,
            set_single_shift3,
        )),
    ))
    .parse_next(input)
}

pub fn parse_escape(input: &str) -> IResult<&str, AnsiSequence> {
    preceded("\u{1b}", combined).parse_peek(input)
}
