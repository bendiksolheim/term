use crate::ansi_parser::{self, AnsiParser, CSISequence, ESCSequence};
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
                    self.buffer_mut().set(c, current_cell_style);
                }),

                ansi_parser::Output::AnsiSequence(code) => match code {
                    ansi_parser::AnsiSequence::OSC(_osc) => {
                        // do nothing as of now
                    }
                    ansi_parser::AnsiSequence::ESC(esc) => match esc {
                        ESCSequence::SetAlternateKeypad | ESCSequence::SetNumericKeypad => {
                            // We don’t support keypad right now
                        }
                        _ => {
                            println!("Unknown ESC code: {:?}", esc);
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
                            self.buffer_mut().cursor.up(n.try_into().unwrap());
                        }

                        CSISequence::CursorDown(n) => {
                            self.buffer_mut().cursor.down(n.try_into().unwrap());
                        }

                        CSISequence::CursorForward(n) => {
                            self.buffer_mut().cursor.right(usize::try_from(n).unwrap());
                        }

                        CSISequence::CursorBackward(n) => {
                            self.buffer_mut().cursor.left(usize::try_from(n).unwrap());
                        }

                        CSISequence::CursorSave => {
                            self.buffer_mut().save_cursor();
                        }

                        CSISequence::CursorRestore => {
                            self.buffer_mut().restore_cursor();
                        }

                        CSISequence::EraseDisplay(n) => {
                            let cursor = self.buffer().cursor.clone();
                            self.buffer_mut().clear_selection(Selection::ToEndOfDisplay(cursor));
                        }

                        CSISequence::EraseLine => {
                            let cursor = self.buffer().cursor.clone();
                            self.buffer_mut().clear_selection(Selection::ToEndOfLine(cursor));
                        }

                        CSISequence::SetGraphicsMode(styles) => {
                            self.current_cell_style.modify(&styles);
                        }

                        CSISequence::HideCursor => {
                            self.cursor_visible = false;
                        }

                        CSISequence::ShowCursor => {
                            self.cursor_visible = true;
                        }

                        CSISequence::CursorToApp => {
                            self.application_mode = true;
                        }

                        CSISequence::SetCursorKeyToCursor => {
                            self.application_mode = false;
                        }

                        CSISequence::SetNewLineMode => {
                            self.newline_mode = true;
                        }

                        CSISequence::SetLineFeedMode => {
                            self.newline_mode = false;
                        }

                        CSISequence::EnableBracketedPasteMode => {
                            // TODO: Must be implemented before pasting
                        }

                        CSISequence::DisableBracketedPasteMode => {
                            // TODO: Must be implemented before pasting
                        }

                        CSISequence::ShowAlternateBuffer => {
                            let rows = self.buffer().rows;
                            let cols = self.buffer().cols;
                            self.alternate_buffer = Some(Buffer::new(rows, cols, vec![Cell::default(); rows * cols]))
                        }

                        CSISequence::ShowNormalBuffer => {
                            self.alternate_buffer = None;
                        }

                        CSISequence::EnableFocusMode => {
                            self.focus_mode = true;
                        }

                        CSISequence::DisableFocusMode => {
                            self.focus_mode = false;
                        }

                        _ => {
                            println!("Unknown CSI code: {:?}", csi);
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
