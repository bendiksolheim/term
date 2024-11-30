use crate::ansi_parser::ansi_sequences::Output;
use crate::ansi_parser::parser::parse_sequence;

pub trait AnsiParser {
    fn ansi_parse(&self) -> AnsiParseIterator<'_>;
}

impl AnsiParser for str {
    fn ansi_parse(&self) -> AnsiParseIterator<'_> {
        AnsiParseIterator { dat: self }
    }
}

impl AnsiParser for String {
    fn ansi_parse(&self) -> AnsiParseIterator<'_> {
        AnsiParseIterator { dat: self }
    }
}

#[derive(Debug)]
pub struct AnsiParseIterator<'a> {
    dat: &'a str,
}

impl<'a> Iterator for AnsiParseIterator<'a> {
    type Item = Output<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.dat.is_empty() {
            return None;
        }

        let pos = self.dat.find('\u{1b}');
        if let Some(loc) = pos {
            if loc == 0 {
                let res = parse_sequence(&self.dat[loc..]);

                if let Ok(ret) = res {
                    self.dat = ret.0;
                    Some(Output::AnsiSequence(ret.1))
                } else {
                    let pos = self.dat[(loc + 1)..].find('\u{1b}');
                    if let Some(loc) = pos {
                        //Added to because it's based one character ahead
                        let loc = loc + 1;
                        println!("Possible undetected escape code in sequence: {:?}", [..loc]);

                        let temp = &self.dat[..loc];
                        self.dat = &self.dat[loc..];

                        Some(Output::TextBlock(temp))
                    } else {
                        println!("Possible undetected escape code in sequence: {:?}", self.dat);
                        let temp = self.dat;
                        self.dat = "";

                        Some(Output::TextBlock(temp))
                    }
                }
            } else {
                let temp = &self.dat[..loc];
                self.dat = &self.dat[loc..];

                Some(Output::TextBlock(temp))
            }
        } else {
            let temp = self.dat;
            self.dat = "";
            Some(Output::TextBlock(temp))
        }
    }
}
