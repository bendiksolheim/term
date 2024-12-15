use crate::{
    ansi_parser::{self, AnsiParser, CSISequence, ESCSequence},
    structs::cursor::Direction,
};
use iced::{
    futures::{channel::mpsc, SinkExt},
    Task,
};
use rustix_openpty::rustix::termios::Winsize;

use crate::{
    structs::{
        buffer::{Buffer, Selection},
        cell::{Cell, CellStyle},
        terminalsize::TerminalSize,
    },
    term::{
        self,
        term::{Event, TermMessage},
        terminal_output::TerminalOutput,
    },
    Message,
};

pub struct Terminal {
    application_mode: bool, // Changes how cursor keys are coded
    newline_mode: bool,     // Interprets \n as NL LF instead of just NL
    focus_mode: bool,       // When enabled, sends \e[I on focus and \e[O on defocus
    auto_wrap_mode: bool,   // Automatically wraps to next line when cursor is at end of line
    size: TerminalSize,
    cursor_visible: bool,
    buffer: Buffer<Cell>,
    alternate_buffer: Option<Buffer<Cell>>,
    current_cell_style: CellStyle,
    sender: Option<mpsc::Sender<term::term::TermMessage>>,
}

impl Terminal {
    pub fn new(size: TerminalSize) -> Self {
        let cols = size.cols as usize;
        let rows = size.rows as usize;

        Self {
            application_mode: false,
            newline_mode: false,
            focus_mode: false,
            auto_wrap_mode: true,
            size,
            cursor_visible: true,
            buffer: Buffer::new(rows, cols, vec![Cell::default(); rows * cols]),
            alternate_buffer: None,
            current_cell_style: CellStyle::default(),
            sender: None,
        }
    }

    pub fn buffer(&self) -> &Buffer<Cell> {
        if let Some(buffer) = &self.alternate_buffer {
            buffer
        } else {
            &self.buffer
        }
    }

    fn buffer_mut(&mut self) -> &mut Buffer<Cell> {
        if let Some(buffer) = &mut self.alternate_buffer {
            buffer
        } else {
            &mut self.buffer
        }
    }

    pub fn send(&self, message: TermMessage) -> Task<Message> {
        if let Some(sender) = self.sender.clone() {
            let f = async move {
                let mut sender = sender;
                sender.send(message).await.expect("Could not send TermMessage");
            };
            Task::perform(f, |_| Message::TerminalInput)
        } else {
            Task::none()
        }
    }

    pub fn parse(&mut self, event: Event) -> Task<Message> {
        match event {
            term::term::Event::Ready(sender) => {
                self.sender = Some(sender);
                Task::none()
            }
            term::term::Event::Output(output) => {
                for token in output {
                    match token {
                        TerminalOutput::Text(s) => {
                            self.handle_ansi(&s);
                        }
                        TerminalOutput::NewLine => {
                            let newline_mode = self.newline_mode.clone();
                            self.buffer_mut().newline(newline_mode);
                        }
                        TerminalOutput::CarriageReturn => {
                            self.buffer_mut().carriage_return();
                        }
                        TerminalOutput::Backspace => {
                            self.buffer_mut().backspace();
                        }
                    }
                }
                Task::none()
            }
        }
    }

    fn handle_ansi(&mut self, ansi_text: &str) {
        let parsed = ansi_text.ansi_parse();
        for block in parsed.into_iter() {
            match block {
                ansi_parser::Output::TextBlock(text) => text.chars().for_each(|c| {
                    let current_cell_style = self.current_cell_style.clone();
                    let auto_wrap_mode = self.auto_wrap_mode;
                    self.buffer_mut().write(c, current_cell_style);
                    self.buffer_mut().advance_cursor(auto_wrap_mode);
                }),

                ansi_parser::Output::AnsiSequence(code) => match code {
                    ansi_parser::AnsiSequence::OSC(_osc) => {
                        // do nothing as of now
                    }

                    ansi_parser::AnsiSequence::ESC(esc) => match esc {
                        ESCSequence::SetAlternateKeypad | ESCSequence::SetNumericKeypad => {
                            // We don’t support keypad right now
                        }
                        ESCSequence::SetUSG0 => {
                            // Don’t do anything, we assume US ASCII is active
                        }

                        ESCSequence::ReverseIndex => {
                            self.buffer_mut().unshift_row();
                        }

                        _ => {
                            println!("Unimplemented ESC code: {:?}", esc);
                        }
                    },

                    ansi_parser::AnsiSequence::CSI(csi) => match csi {
                        CSISequence::CursorPos(row, col) => {
                            // Cursor position starts at 1,1 in terminal, while grid starts at 0,0
                            let grid_row = (row - 1) as usize;
                            let grid_col = (col - 1) as usize;
                            self.buffer_mut().cursor.set_position(grid_row, grid_col);
                        }

                        CSISequence::CursorUp(n) => {
                            self.buffer_mut().move_cursor(Direction::Up(n.try_into().unwrap()));
                        }

                        CSISequence::CursorDown(n) => {
                            self.buffer_mut().move_cursor(Direction::Down(n.try_into().unwrap()));
                        }

                        CSISequence::CursorForward(n) => {
                            self.buffer_mut().move_cursor(Direction::Right(n.try_into().unwrap()));
                        }

                        CSISequence::CursorBackward(n) => {
                            self.buffer_mut().move_cursor(Direction::Left(n.try_into().unwrap()));
                        }

                        CSISequence::LinePositionAbsolute(n) => {
                            self.buffer_mut().cursor.row = n as usize - 1;
                        }

                        CSISequence::CursorCharacterAbsolute(n) => {
                            self.buffer_mut().cursor.col = n as usize - 1;
                        }

                        CSISequence::CursorSave => {
                            self.buffer_mut().save_cursor();
                        }

                        CSISequence::CursorRestore => {
                            self.buffer_mut().restore_cursor();
                        }

                        CSISequence::EraseDisplay(n) => {
                            self.buffer_mut().clear_selection(Selection::ToEndOfDisplay);
                        }

                        CSISequence::EraseInLine(n) => {
                            let selection = match n {
                                0 => Selection::ToEndOfLine,
                                1 => Selection::FromStartOfLine,
                                2 => Selection::Line,
                                _ => unreachable!(),
                            };
                            self.buffer_mut().clear_selection(selection);
                        }

                        CSISequence::EraseCharacters(n) => {
                            self.buffer_mut().clear_selection(Selection::Characters(n));
                        }

                        CSISequence::SetGraphicsMode(styles) => {
                            self.current_cell_style.modify(&styles);
                        }

                        CSISequence::DecPrivateModeSet(n) => match n {
                            1 => self.application_mode = true,
                            7 => self.auto_wrap_mode = true,
                            25 => self.cursor_visible = false,
                            1004 => self.focus_mode = true,
                            1049 => {
                                let rows = self.buffer().rows;
                                let cols = self.buffer().cols;
                                self.alternate_buffer =
                                    Some(Buffer::new(rows, cols, vec![Cell::default(); rows * cols]))
                            }
                            n => println!("Unimplemented DecPrivateModeSet value {}", n),
                        },

                        CSISequence::DecPrivateModeReset(n) => match n {
                            1 => self.application_mode = false,
                            7 => self.auto_wrap_mode = false,
                            25 => self.cursor_visible = true,
                            1004 => self.focus_mode = false,
                            1049 => self.alternate_buffer = None,
                            n => println!("Unimplemented DecPrivateModeReset value {}", n),
                        },

                        CSISequence::SetNewLineMode => {
                            self.newline_mode = true;
                        }

                        CSISequence::SetLineFeedMode => {
                            self.newline_mode = false;
                        }

                        CSISequence::CursorStyle(style) => {
                            self.buffer_mut().cursor.set_style(style);
                        }

                        CSISequence::SetTopAndBottom(top, bottom) => {
                            self.buffer_mut().set_top_bottom(top as usize, bottom as usize);
                        }

                        _ => {
                            println!("Unimplemented CSI code: {:?}", csi);
                        }
                    },
                },
            }
        }
    }

    pub fn focus(&self) -> Task<Message> {
        if self.focus_mode {
            self.send(TermMessage::Bytes("\x1b[I".into()))
        } else {
            Task::none()
        }
    }

    pub fn unfocus(&self) -> Task<Message> {
        if self.focus_mode {
            self.send(TermMessage::Bytes("\x1b[O".into()))
        } else {
            Task::none()
        }
    }

    pub fn resize(&mut self, new_size: TerminalSize) -> Task<Message> {
        self.buffer_mut().resize(new_size.rows, new_size.cols);
        self.send(TermMessage::WindowResized(new_size.cols, new_size.rows))
    }

    pub fn winsize(&self) -> Winsize {
        self.size.winsize()
    }
}
