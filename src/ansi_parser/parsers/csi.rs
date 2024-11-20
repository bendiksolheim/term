use crate::ansi_parser::{AnsiSequence, CSISequence};

use heapless::Vec;
use winnow::ascii::{digit0, digit1};
use winnow::combinator::{alt, delimited, opt, preceded, terminated};
use winnow::error::InputError;
use winnow::token::literal;
use winnow::{PResult, Parser};

macro_rules! tag_parser {
    ($sig:ident, $tag:expr, $ret:expr) => {
        fn $sig<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
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

fn cursor_pos<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (parse_def_cursor_int, opt(";"), parse_def_cursor_int, alt(("H", "f")))
        .map(|(x, _, y, _)| CSISequence::CursorPos(x, y))
        .parse_next(input)
}

fn cursor_up<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    terminated(parse_def_cursor_int, "A")
        .map(|am: u32| CSISequence::CursorUp(am))
        .parse_next(input)
}

fn cursor_down<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    terminated(parse_def_cursor_int, "B")
        .map(|am| CSISequence::CursorDown(am))
        .parse_next(input)
}

fn cursor_forward<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    terminated(parse_def_cursor_int, "C")
        .map(|am| CSISequence::CursorForward(am))
        .parse_next(input)
}

fn cursor_backward<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    terminated(parse_def_cursor_int, "D")
        .map(|am| CSISequence::CursorBackward(am))
        .parse_next(input)
}

fn cursor_style<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (parse_u8, " ", "q")
        .map(|(style, _, _)| CSISequence::CursorStyle(style))
        .parse_next(input)
}

fn graphics_mode1<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    terminated(parse_u8, "m")
        .map(|val| {
            let mode = Vec::from_slice(&[val]).expect("Vec::from_slice should allocate sufficient size");
            CSISequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode2<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (parse_u8, ";", parse_u8, "m")
        .map(|(val1, _, val2, _)| {
            let mode = Vec::from_slice(&[val1, val2]).expect("Vec::from_slice should allocate sufficient size");
            CSISequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode3<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (parse_u8, ";", parse_u8, ";", parse_u8, "m")
        .map(|(val1, _, val2, _, val3, _)| {
            let mode = Vec::from_slice(&[val1, val2, val3]).expect("Vec::from_slice should allocate sufficient size");
            CSISequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode4<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    "m".value(CSISequence::SetGraphicsMode(Vec::new())).parse_next(input)
}

fn graphics_mode5<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (
        parse_u8, ";", parse_u8, ";", parse_u8, ";", parse_u8, ";", parse_u8, "m",
    )
        .map(|(val1, _, val2, _, val3, _, val4, _, val5, _)| {
            let mode = Vec::from_slice(&[val1, val2, val3, val4, val5])
                .expect("Vec::from_slice should allocate sufficient size");
            CSISequence::SetGraphicsMode(mode)
        })
        .parse_next(input)
}

fn graphics_mode<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    alt((
        graphics_mode1,
        graphics_mode2,
        graphics_mode3,
        graphics_mode4,
        graphics_mode5,
    ))
    .parse_next(input)
}

fn set_mode<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    delimited("=", parse_u8, "h")
        .map(|val| CSISequence::SetMode(val))
        .parse_next(input)
}

fn reset_mode<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    delimited("=", parse_u8, "l")
        .map(|val| CSISequence::ResetMode(val))
        .parse_next(input)
}

fn set_top_and_bottom<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (parse_u32, ";", parse_u32, "r")
        .map(|(x, _, y, _)| CSISequence::SetTopAndBottom(x, y))
        .parse_next(input)
}

fn erase_display<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    terminated(digit0.map(|s: &str| s.parse::<u8>().unwrap_or(0)), "J")
        .map(|n| CSISequence::EraseDisplay(n))
        .parse_next(input)
}

fn erase_characters<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    (parse_def_cursor_int, "X")
        .map(|(n, _)| CSISequence::EraseCharacters(n))
        .parse_next(input)
}

tag_parser!(cursor_save, "s", CSISequence::CursorSave);
tag_parser!(cursor_restore, "u", CSISequence::CursorRestore);
tag_parser!(erase_line, "K", CSISequence::EraseLine);
tag_parser!(hide_cursor, "?25l", CSISequence::HideCursor);
tag_parser!(show_cursor, "?25h", CSISequence::ShowCursor);
tag_parser!(cursor_to_app, "?1h", CSISequence::CursorToApp);
tag_parser!(set_new_line_mode, "20h", CSISequence::SetNewLineMode);
tag_parser!(set_col_132, "?3h", CSISequence::SetCol132);
tag_parser!(set_smooth_scroll, "?4h", CSISequence::SetSmoothScroll);
tag_parser!(set_reverse_video, "?5h", CSISequence::SetReverseVideo);
tag_parser!(set_origin_rel, "?6h", CSISequence::SetOriginRelative);
tag_parser!(set_auto_wrap, "?7h", CSISequence::SetAutoWrap);
tag_parser!(set_auto_repeat, "?8h", CSISequence::SetAutoRepeat);
tag_parser!(set_interlacing, "?9h", CSISequence::SetInterlacing);
tag_parser!(set_linefeed, "20l", CSISequence::SetLineFeedMode);
tag_parser!(set_cursorkey, "?1l", CSISequence::SetCursorKeyToCursor);
tag_parser!(set_vt52, "?2l", CSISequence::SetVT52);
tag_parser!(set_col80, "?3l", CSISequence::SetCol80);
tag_parser!(set_jump_scroll, "?4l", CSISequence::SetJumpScrolling);
tag_parser!(set_normal_video, "?5l", CSISequence::SetNormalVideo);
tag_parser!(set_origin_abs, "?6l", CSISequence::SetOriginAbsolute);
tag_parser!(reset_auto_wrap, "?7l", CSISequence::ResetAutoWrap);
tag_parser!(reset_auto_repeat, "?8l", CSISequence::ResetAutoRepeat);
tag_parser!(reset_interlacing, "?9l", CSISequence::ResetInterlacing);
tag_parser!(
    enable_motion_mouse_tracking,
    "?1002h",
    CSISequence::EnableMotionMouseTracking
);
tag_parser!(
    disable_motion_mouse_tracking,
    "?1002l",
    CSISequence::DisableMotionMouseTracking
);
tag_parser!(enable_focus_mode, "?1004h", CSISequence::EnableFocusMode);
tag_parser!(disable_focus_mode, "?1004l", CSISequence::DisableFocusMode);
tag_parser!(enable_sgr_mouse_mode, "?1006h", CSISequence::EnableSGRMouseMode);
tag_parser!(disable_sgr_mouse_mode, "?1006l", CSISequence::DisableSGRMouseMode);
tag_parser!(set_alternate_buffer, "?1049h", CSISequence::ShowAlternateBuffer);
tag_parser!(set_normal_buffer, "?1049l", CSISequence::ShowNormalBuffer);
tag_parser!(
    enable_bracketed_paste_mode,
    "?2004h",
    CSISequence::EnableBracketedPasteMode
);
tag_parser!(
    disable_bracketed_paste_mode,
    "?2004l",
    CSISequence::DisableBracketedPasteMode
);

fn combined<'s>(input: &mut &'s str) -> PResult<CSISequence, InputError<&'s str>> {
    // `alt` only supports up to 21 parsers, and winnow doesn't seem to
    // have an alternative with higher variability.
    // So we simply nest them.
    alt((
        alt((
            cursor_pos,
            cursor_up,
            cursor_down,
            cursor_forward,
            cursor_backward,
            cursor_style,
            cursor_save,
            cursor_restore,
            erase_display,
            erase_line,
            erase_characters,
            graphics_mode,
            set_mode,
            reset_mode,
            hide_cursor,
            show_cursor,
            cursor_to_app,
            set_new_line_mode,
            set_col_132,
            set_smooth_scroll,
        )),
        alt((
            set_reverse_video,
            set_origin_rel,
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
            enable_motion_mouse_tracking,
            disable_motion_mouse_tracking,
            enable_focus_mode,
            disable_focus_mode,
            enable_sgr_mouse_mode,
        )),
        alt((
            disable_sgr_mouse_mode,
            enable_bracketed_paste_mode,
            disable_bracketed_paste_mode,
            set_alternate_buffer,
            set_normal_buffer,
            set_top_and_bottom,
        )),
    ))
    .parse_next(input)
}

pub fn parse_csi_sequence<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    preceded("\u{1b}[", combined)
        .map(|a| AnsiSequence::CSI(a))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::ansi_parser::ansi_sequences::{AnsiSequence, CSISequence, Output};
    use crate::ansi_parser::parser::parse_sequence;
    use crate::ansi_parser::traits::AnsiParser;
    use std::fmt::Write;

    macro_rules! test_parser {
        ($name:ident, $string:expr) => {
            #[test]
            fn $name() {
                let mut buff = String::new();
                let ret = parse_sequence($string);

                assert!(ret.is_ok());
                let ret = ret.unwrap().1;

                write!(&mut buff, "{}", ret).unwrap();

                assert_eq!(buff, $string);
            }
        };
    }

    macro_rules! test_def_val_parser {
        ($name:ident, $string:expr) => {
            #[test]
            fn $name() {
                let mut buff = String::new();
                let ret = parse_sequence($string);

                assert!(ret.is_ok());
                let ret = ret.unwrap().1;

                write!(&mut buff, "{}", ret).unwrap();

                let ret2 = parse_sequence(&buff);
                assert!(ret2.is_ok());

                let ret2 = ret2.unwrap().1;
                assert_eq!(ret, ret2);
            }
        };
    }

    test_def_val_parser!(cursor_pos_default, "\u{1b}[H");
    test_def_val_parser!(cursor_pos, "\u{1b}[10;5H");
    test_def_val_parser!(cursor_up_default, "\u{1b}[A");
    test_def_val_parser!(cursor_up, "\u{1b}[5A");
    test_def_val_parser!(cursor_down, "\u{1b}[5B");
    test_def_val_parser!(cursor_forward, "\u{1b}[5C");
    test_def_val_parser!(cursor_backward, "\u{1b}[5D");
    test_parser!(cursor_block_style, "\u{1b}[2 q");
    test_parser!(cursor_save, "\u{1b}[s");
    test_parser!(cursor_restore, "\u{1b}[u");

    test_parser!(erase_display_a, "\u{1b}[J");
    test_parser!(erase_display_b, "\u{1b}[1J");
    test_parser!(erase_display_c, "\u{1b}[2J");
    test_parser!(erase_line, "\u{1b}[K");
    test_parser!(erase_characters, "\u{1b}[43X");

    test_parser!(set_video_mode_a, "\u{1b}[4m");
    test_parser!(set_video_mode_b, "\u{1b}[4;42m");
    test_parser!(set_video_mode_c, "\u{1b}[4;31;42m");
    test_parser!(set_video_mode_d, "\u{1b}[4;31;42;42;42m");

    test_parser!(reset_mode, "\u{1b}[=13l");
    test_parser!(set_mode, "\u{1b}[=7h");

    test_parser!(show_cursor, "\u{1b}[?25h");
    test_parser!(hide_cursor, "\u{1b}[?25l");
    test_parser!(cursor_to_app, "\u{1b}[?1h");

    test_parser!(set_newline_mode, "\u{1b}[20h");
    test_parser!(set_column_132, "\u{1b}[?3h");
    test_parser!(set_smooth_scroll, "\u{1b}[?4h");
    test_parser!(set_reverse_video, "\u{1b}[?5h");
    test_parser!(set_origin_rel, "\u{1b}[?6h");
    test_parser!(set_auto_wrap, "\u{1b}[?7h");
    test_parser!(set_auto_repeat, "\u{1b}[?8h");
    test_parser!(set_interlacing, "\u{1b}[?9h");
    test_parser!(enable_focus_mode, "\u{1b}[?1004h");
    test_parser!(disable_focus_mode, "\u{1b}[?1004l");
    test_parser!(enable_bracketed_paste_mode, "\u{1b}[?2004h");
    test_parser!(disable_bracketed_paste_mode, "\u{1b}[?2004l");

    test_parser!(set_cursor_key_to_cursor, "\u{1b}[?1l");

    test_parser!(set_linefeed, "\u{1b}[20l");
    test_parser!(set_vt52, "\u{1b}[?2l");
    test_parser!(set_col80, "\u{1b}[?3l");
    test_parser!(set_jump_scroll, "\u{1b}[?4l");
    test_parser!(set_normal_video, "\u{1b}[?5l");
    test_parser!(set_origin_abs, "\u{1b}[?6l");
    test_parser!(reset_auto_wrap, "\u{1b}[?7l");
    test_parser!(reset_auto_repeat, "\u{1b}[?8l");
    test_parser!(reset_interlacing, "\u{1b}[?9l");
    test_parser!(enable_motion_mouse_tracking, "\u{1b}[?1002h");
    test_parser!(disable_motion_mouse_tracking, "\u{1b}[?1002l");
    test_parser!(enable_sgr_mouse_mode, "\u{1b}[?1006h");
    test_parser!(disable_sgr_mouse_mode, "\u{1b}[?1006l");

    #[test]
    fn test_parser_iterator() {
        let count = "\x1b[=25l\x1b[=7l\x1b[0m\x1b[36m\x1b[1m-`".ansi_parse().count();

        assert_eq!(count, 6);
    }

    #[test]
    fn test_parser_iterator_failure() {
        let count = "\x1b[=25l\x1b[=7l\x1b[0m\x1b[36;1;15;2m\x1b[1m-`".ansi_parse().count();

        assert_eq!(count, 6);
    }

    #[test]
    fn test_default_value() {
        let strings: Vec<_> = "\x1b[H\x1b[123456H\x1b[;123456H\x1b[7asd;1234H\x1b[a;sd7H"
            .ansi_parse()
            .collect();
        assert_eq!(strings.len(), 5);
        assert_eq!(
            strings[0],
            Output::AnsiSequence(AnsiSequence::CSI(CSISequence::CursorPos(1, 1)))
        );
        assert_eq!(
            strings[1],
            Output::AnsiSequence(AnsiSequence::CSI(CSISequence::CursorPos(123456, 1)))
        );
        assert_eq!(
            strings[2],
            Output::AnsiSequence(AnsiSequence::CSI(CSISequence::CursorPos(1, 123456)))
        );
        assert_eq!(strings[3], Output::TextBlock("\x1b[7asd;1234H"));
        assert_eq!(strings[4], Output::TextBlock("\x1b[a;sd7H"));
    }

    #[test]
    fn test_cursor_pos() {
        let pos = CSISequence::CursorPos(5, 20);
        let mut buff = String::new();

        write!(&mut buff, "{}", pos).expect("failed to write");

        assert_eq!(buff, "\x1b[5;20H");
    }
}
