use std::io::{ErrorKind, Read};

use super::terminal_output::TerminalOutput;

pub struct PtyReader<R: Read> {
    inner: R,
    buffer: Vec<u8>,
}

impl<R: Read> PtyReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
        }
    }

    pub fn read_chunk(&mut self) -> PtyReaderResult {
        let mut chunk = [0u8; 1024];
        match self.inner.read(&mut chunk) {
            Ok(n) => {
                self.buffer.extend_from_slice(&chunk[..n]);
                PtyReaderResult::MoreLeft
            }
            Err(e) => {
                // WouldBlock is expected when there is no input
                if e.kind() == ErrorKind::WouldBlock {
                    PtyReaderResult::EndOfInput
                } else {
                    eprintln!("Error reading from PTY: {:?}", e);
                    panic!();
                }
            }
        }
    }

    pub fn process_buffer(&mut self) -> Option<Vec<TerminalOutput>> {
        if self.buffer.len() > 0 {
            let mut byte_sequence: Vec<u8> = vec![];
            let mut output: Vec<TerminalOutput> = vec![];
            for byte in self.buffer.iter() {
                match byte {
                    b'\x08' => {
                        if byte_sequence.len() > 0 {
                            let _s = byte_sequence.drain(0..).collect();
                            let str_sequence = String::from_utf8(_s).unwrap();
                            output.push(TerminalOutput::Text(str_sequence));
                        }
                        output.push(TerminalOutput::Backspace);
                    }
                    b'\n' => {
                        if byte_sequence.len() > 0 {
                            let _s = byte_sequence.drain(0..).collect();
                            let str_sequence = String::from_utf8(_s).unwrap();
                            output.push(TerminalOutput::Text(str_sequence));
                        }
                        output.push(TerminalOutput::NewLine);
                    }
                    b'\r' => {
                        if byte_sequence.len() > 0 {
                            let _s = byte_sequence.drain(0..).collect();
                            let str_sequence = String::from_utf8(_s).unwrap();
                            output.push(TerminalOutput::Text(str_sequence));
                        }
                        output.push(TerminalOutput::CarriageReturn);
                    }
                    b => {
                        byte_sequence.push(*b);
                    }
                }
            }
            let _s = String::from_utf8(byte_sequence.clone()).unwrap();
            output.push(TerminalOutput::Text(_s));

            // Buffer is consumed, clear it
            self.buffer.drain(..);

            Some(output)
        } else {
            None
        }
    }
}

pub enum PtyReaderResult {
    MoreLeft,
    EndOfInput,
}
