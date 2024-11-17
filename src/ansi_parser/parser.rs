use winnow::combinator::alt;
use winnow::{IResult, Parser};

use crate::ansi_parser::parsers::csi::parse_csi_sequence;
use crate::ansi_parser::parsers::esc::parse_esc_sequence;
use crate::ansi_parser::parsers::osc::parse_osc_sequence;

use super::AnsiSequence;

pub fn parse_sequence(input: &str) -> IResult<&str, AnsiSequence> {
    alt((parse_csi_sequence, parse_osc_sequence, parse_esc_sequence)).parse_peek(input)
}
