use crate::ansi_parser::{AnsiSequence, ESCSequence};

use winnow::combinator::{alt, preceded};
use winnow::error::InputError;
use winnow::token::literal;
use winnow::{PResult, Parser};

macro_rules! tag_parser {
    ($sig:ident, $tag:expr, $ret:expr) => {
        fn $sig<'s>(input: &mut &'s str) -> PResult<ESCSequence, InputError<&'s str>> {
            literal($tag).value($ret).parse_next(input)
        }
    };
}

tag_parser!(set_alternate_keypad, "=", ESCSequence::SetAlternateKeypad);
tag_parser!(set_numeric_keypad, ">", ESCSequence::SetNumericKeypad);
tag_parser!(set_single_shift2, "N", ESCSequence::SetSingleShift2);
tag_parser!(set_single_shift3, "O", ESCSequence::SetSingleShift3);
tag_parser!(set_uk_g0, "(A", ESCSequence::SetUKG0);
tag_parser!(set_uk_g1, ")A", ESCSequence::SetUKG1);
tag_parser!(set_us_g0, "(B", ESCSequence::SetUSG0);
tag_parser!(set_us_g1, ")B", ESCSequence::SetUSG1);
tag_parser!(set_g0_special, "(0", ESCSequence::SetG0SpecialChars);
tag_parser!(set_g1_special, ")0", ESCSequence::SetG1SpecialChars);
tag_parser!(set_g0_alternate, "(1", ESCSequence::SetG0AlternateChar);
tag_parser!(set_g1_alternate, ")1", ESCSequence::SetG1AlternateChar);
tag_parser!(set_g0_graph, "(2", ESCSequence::SetG0AltAndSpecialGraph);
tag_parser!(set_g1_graph, ")2", ESCSequence::SetG1AltAndSpecialGraph);

fn combined<'s>(input: &mut &'s str) -> PResult<ESCSequence, InputError<&'s str>> {
    alt((
        set_alternate_keypad,
        set_numeric_keypad,
        set_single_shift2,
        set_single_shift3,
        set_uk_g0,
        set_uk_g1,
        set_us_g0,
        set_us_g1,
        set_g0_special,
        set_g1_special,
        set_g0_alternate,
        set_g1_alternate,
        set_g0_graph,
        set_g1_graph,
    ))
    .parse_next(input)
}

pub fn parse_esc_sequence<'s>(input: &mut &'s str) -> PResult<AnsiSequence, InputError<&'s str>> {
    preceded("\u{1b}", combined)
        .map(|a| AnsiSequence::ESC(a))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::ansi_parser::parser::parse_sequence;
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

    macro_rules! _test_def_val_parser {
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

    test_parser!(set_single_shift2, "\u{1b}N");
    test_parser!(set_single_shift3, "\u{1b}O");

    test_parser!(set_alternate_keypad, "\u{1b}=");
    test_parser!(set_numeric_keypad, "\u{1b}>");
    test_parser!(set_uk_g0, "\u{1b}(A");
    test_parser!(set_uk_g1, "\u{1b})A");
    test_parser!(set_us_g0, "\u{1b}(B");
    test_parser!(set_us_g1, "\u{1b})B");
    test_parser!(set_g0_special, "\u{1b}(0");
    test_parser!(set_g1_special, "\u{1b})0");
    test_parser!(set_g0_alternate, "\u{1b}(1");
    test_parser!(set_g1_alternate, "\u{1b})1");
    test_parser!(set_g0_graph, "\u{1b}(2");
    test_parser!(set_g1_graph, "\u{1b})2");
}
