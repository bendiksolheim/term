use crate::ansi_parser::{self, AnsiParser};
use iced::{futures::{channel::mpsc, SinkExt}, Task};

use crate::{structs::{cell::{Cell, CellStyle}, cursor::Cursor, grid::{Grid, Selection}, terminalsize::TerminalSize}, term::{self, term::{Event, TermMessage}, terminal_output::TerminalOutput}, Message};

pub struct Terminal {
    application_mode: bool, // Changes how cursor keys are coded
    newline_mode: bool,     // Interprets \n as NL LF instead of just NL
    pub size: TerminalSize,
    pub cursor: Cursor,
    cursor_visible: bool,
    saved_cursor_position: Option<Cursor>,
    pub content: Grid<Cell>,
    current_cell_style: CellStyle,
    pub sender: Option<mpsc::Sender<term::term::TermMessage>>,
}

impl Terminal {
    pub fn new(size: TerminalSize) -> Self {
        let cols = size.cols as usize;
        let rows = size.rows as usize;
        Self {
            application_mode: false,
            newline_mode: false,
            size,
            cursor: Cursor::default(),
            cursor_visible: true,
            saved_cursor_position: None,
            content: Grid::new(rows, cols, vec![Cell::default(); rows * cols]),
            current_cell_style: CellStyle:: default(),
            sender: None,
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
                            if self.cursor.row == self.size.rows - 1 {
                                self.content.shift_row();
                            } else {
                                self.cursor.down(1);
                            }

                            // If terminal is in newline mode, cursor is also moved to start of line
                            if self.newline_mode {
                                self.cursor.col = 0;
                            }
                        }
                        TerminalOutput::CarriageReturn => {
                            self.cursor.col = 0;
                        }
                        TerminalOutput::Backspace => {
                            self.cursor.left(1);
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
                    self.content[self.cursor].content = c;
                    self.content[self.cursor].style = self.current_cell_style.clone();
                    self.cursor.right(1);
                }),
                ansi_parser::Output::Escape(code) => match code {
                    ansi_parser::AnsiSequence::CursorPos(row, col) => {
                        self.cursor
                            .set_position(row.try_into().unwrap(), col.try_into().unwrap());
                    }

                    ansi_parser::AnsiSequence::CursorUp(n) => {
                        self.cursor.up(n.try_into().unwrap());
                    }

                    ansi_parser::AnsiSequence::CursorDown(n) => {
                        self.cursor.down(n.try_into().unwrap());
                    }

                    ansi_parser::AnsiSequence::CursorForward(n) => {
                        self.cursor.right(usize::try_from(n).unwrap());
                    }

                    ansi_parser::AnsiSequence::CursorBackward(n) => {
                        self.cursor.left(usize::try_from(n).unwrap());
                    }

                    ansi_parser::AnsiSequence::CursorSave => {
                        self.saved_cursor_position = Some(self.cursor.clone());
                    }

                    ansi_parser::AnsiSequence::CursorRestore => {
                        if let Some(cursor) = self.saved_cursor_position {
                            self.cursor = cursor;
                            self.saved_cursor_position = None;
                        }
                    }

                    ansi_parser::AnsiSequence::EraseDisplay(n) => {
                        self.content.clear_selection(Selection::ToEndOfDisplay(self.cursor));
                    }

                    ansi_parser::AnsiSequence::EraseLine => {
                        self.content.clear_selection(Selection::ToEndOfLine(self.cursor));
                    }

                    ansi_parser::AnsiSequence::SetGraphicsMode(styles) => {
                        self.current_cell_style.modify(&styles);
                    }

                    ansi_parser::AnsiSequence::HideCursor => {
                        self.cursor_visible = false;
                    }

                    ansi_parser::AnsiSequence::ShowCursor => {
                        self.cursor_visible = true;
                    }

                    ansi_parser::AnsiSequence::CursorToApp => {
                        self.application_mode = true;
                    }

                    ansi_parser::AnsiSequence::SetCursorKeyToCursor => {
                        self.application_mode = false;
                    }

                    ansi_parser::AnsiSequence::SetNewLineMode => {
                        self.newline_mode = true;
                    }

                    ansi_parser::AnsiSequence::SetLineFeedMode => {
                        self.newline_mode = false;
                    }

                    ansi_parser::AnsiSequence::EnableBracketedPasteMode => {
                        // TODO: Must be implemented before pasting
                    }

                    ansi_parser::AnsiSequence::DisableBracketedPasteMode => {
                        // TODO: Must be implemented before pasting
                    }

                    _ => {
                        println!("Unknown escape code: {:?}", code);
                    }
                },
            }
        }
    }
}
