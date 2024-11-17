use crate::ansi_parser::{AnsiSequence, OSCSequence};

use winnow::combinator::{alt, preceded};
use winnow::error::InputError;
use winnow::token::literal;
use winnow::{PResult, Parser};

macro_rules! tag_parser {
    ($sig:ident, $tag:expr, $ret:expr) => {
        fn $sig<'s>(input: &mut &'s str) -> PResult<OSCSequence, InputError<&'s str>> {
            literal($tag).value($ret).parse_next(input)
        }
    };
}

tag_parser!(reset_text_cursor_color, "]112", OSCSequence::ResetCursorColor);

fn combined<'s>(input: &mut &'s str) -> PResult<OSCSequence, InputError<&'s str>> {
    alt([reset_text_cursor_color]).parse_next(input)
}

pub fn parse_osc_sequence<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    preceded("\u{1b}", combined)
        .map(|a| AnsiSequence::OSC(a))
        .parse_next(input)
}
