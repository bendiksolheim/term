mod structs {
    pub mod cell;
    pub mod cursor;
    pub mod grid;
    pub mod terminalsize;
}
mod terminal {
    pub mod colors;
    pub mod graphics;
    pub mod pty_reader;
    pub mod term;
    pub mod terminal_output;
}
mod debug {
    pub mod view;
}

use std::collections::BTreeMap;

use ansi_parser::AnsiParser;
use debug::view::DebugState;
use iced::{
    futures::{
        channel::mpsc::{self},
        SinkExt,
    },
    keyboard::{self, Key, Modifiers},
    widget::{container, text, Column, Row},
    window::{settings::PlatformSpecific, Id, Settings},
    Element, Font, Padding, Subscription, Task,
};
use structs::{
    cell::{Cell, CellStyle},
    cursor::Cursor,
    grid::{Grid, Selection},
    terminalsize::TerminalSize,
};
use terminal::term::Winsize;
use terminal::{colors::TerminalColor, terminal_output::TerminalOutput};

fn main() -> iced::Result {
    iced::daemon("Terminal", Terminalview::update, Terminalview::view)
        .subscription(Terminalview::subscription)
        .run_with(Terminalview::new)
}

pub struct Terminalview {
    size: TerminalSize,
    cursor: Cursor,
    content: Grid<Cell>,
    current_cell_style: CellStyle,
    sender: Option<mpsc::Sender<terminal::term::TermMessage>>,
    windows: BTreeMap<Id, WindowType>,
    debug: DebugState<Message>,
}

enum WindowType {
    TerminalWindow,
    DebugWindow,
}

#[derive(Debug, Clone)]
pub enum Message {
    TerminalInput,
    Keyboard(Key, Modifiers),
    TerminalOutput(terminal::term::Event),
    TerminalWindowVisible(Id),
    DebugWindow(Id),
    ShowMessage(Box<Message>),
}

impl Message {
    fn name(&self) -> &str {
        match self {
            Message::TerminalInput => "TerminalInput",
            Message::Keyboard(_key, _modifiers) => "Keyboard(key, modifiers)",
            Message::TerminalOutput(_event) => "TerminalOutput(event)",
            Message::TerminalWindowVisible(_id) => "TerminalWindowVisible(id)",
            Message::DebugWindow(_id) => "DebugWindow(id)",
            Message::ShowMessage(_message) => "ShowMessage(message)",
        }
    }
}

impl Terminalview {
    fn new() -> (Self, Task<Message>) {
        let (id, terminal_window) = iced::window::open(terminal_window_settings());
        let mut windows = BTreeMap::new();
        windows.insert(id, WindowType::TerminalWindow);
        let size = TerminalSize { cols: 120, rows: 40 };
        let content = Grid::new(size.rows, size.cols, vec![Cell::default(); size.rows * size.cols]);
        let model = Self {
            size,
            cursor: Cursor::default(),
            content,
            current_cell_style: CellStyle::default(),
            sender: None,
            windows,
            debug: DebugState::default(),
        };

        (model, terminal_window.map(|id| Message::TerminalWindowVisible(id)))
    }

    fn view(&self, window: Id) -> Element<'_, Message> {
        match self.windows.get(&window) {
            Some(window) => match window {
                WindowType::TerminalWindow => self.terminal_view(),
                WindowType::DebugWindow => self.debug_view(),
            },
            None => text("").into(),
        }
    }

    fn terminal_view(&self) -> Element<'_, Message> {
        Column::with_children(
            self.content
                .iter_rows()
                .enumerate()
                .map(|(y, row)| {
                    Row::with_children(
                        row.iter()
                            .enumerate()
                            .map(|(x, cell)| cell_view(&self.cursor, x, y, cell))
                            .collect::<Vec<_>>(),
                    )
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .padding(Padding {
            top: 25.0,
            left: 5.0,
            bottom: 5.0,
            right: 5.0,
        })
        .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.debug.messages.push(message.clone());
        match message {
            Message::Keyboard(k, modifiers) => {
                if k == Key::Named(keyboard::key::Named::Enter) && modifiers.command() {
                    let (_id, debug_window) = iced::window::open(debug_window_settings());
                    debug_window.map(|id| Message::DebugWindow(id))
                } else if let Some(sender) = self.sender.clone() {
                    let f = async move {
                        let mut sender = sender;
                        sender
                            .send(terminal::term::TermMessage::Input(k))
                            .await
                            .expect("Could not send TermMessage::Input");
                    };
                    Task::perform(f, |_| Message::TerminalInput)
                } else {
                    Task::none()
                }
            }
            Message::TerminalInput => Task::none(),
            Message::TerminalOutput(term_event) => match term_event {
                terminal::term::Event::Ready(sender) => {
                    self.sender = Some(sender);
                    Task::none()
                }
                terminal::term::Event::Output(output) => {
                    for token in output {
                        match token {
                            TerminalOutput::Text(s) => {
                                self.handle_ansi(&s);
                            }
                            TerminalOutput::NewLine => {
                                if self.cursor.row == self.size.rows - 1 {
                                    self.content.shift_row();
                                } else {
                                    self.cursor.down();
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
            },
            Message::TerminalWindowVisible(id) => {
                self.windows.insert(id, WindowType::TerminalWindow);
                Task::none()
            }
            Message::DebugWindow(id) => {
                self.windows.insert(id, WindowType::DebugWindow);
                Task::none()
            }
            Message::ShowMessage(message) => {
                self.debug.selected = Some(*message);
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
                    ansi_parser::AnsiSequence::SetGraphicsMode(styles) => {
                        self.current_cell_style.modify(styles.into_iter().collect());
                    }

                    ansi_parser::AnsiSequence::CursorForward(n) => {
                        self.cursor.right(usize::try_from(n).unwrap());
                    }

                    ansi_parser::AnsiSequence::CursorBackward(n) => {
                        self.cursor.left(usize::try_from(n).unwrap());
                    }

                    ansi_parser::AnsiSequence::EraseLine => {
                        self.content.clear_selection(Selection::ToEndOfLine(self.cursor));
                    }

                    ansi_parser::AnsiSequence::EraseDisplay(n) => {
                        self.content.clear_selection(Selection::ToEndOfDisplay(self.cursor));
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

    fn subscription(&self) -> Subscription<Message> {
        let keypress = keyboard::on_key_press(|key, modifier| Some(Message::Keyboard(key, modifier)));
        let winsize = create_winsize(self.size);
        let term_sub =
            Subscription::run_with_id(12345, terminal::term::Term::spawn(winsize)).map(Message::TerminalOutput);
        iced::Subscription::batch([term_sub, keypress])
    }
}

fn create_winsize(size: TerminalSize) -> Winsize {
    Winsize {
        ws_col: u16::try_from(size.cols).expect("Terminal is too wide for Winsize"),
        ws_row: u16::try_from(size.rows).expect("Terminal is too tall for Winsize"),
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}

fn terminal_window_settings() -> Settings {
    Settings {
        decorations: true,
        platform_specific: PlatformSpecific {
            title_hidden: true,
            titlebar_transparent: true,
            fullsize_content_view: true,
        },
        ..Settings::default()
    }
}

fn debug_window_settings() -> Settings {
    Settings::default()
}

fn cell_view<'a>(cursor: &Cursor, x: usize, y: usize, cell: &Cell) -> Element<'a, Message> {
    let style = if cursor.col == x && cursor.row == y {
        let mut clone = cell.style.clone();
        clone.background = TerminalColor::White;
        clone.foreground = TerminalColor::Black;
        clone
    } else {
        cell.style.clone()
    };

    let container_style = container::Style {
        // TODO: Do I really need to clone here?
        text_color: Some(style.clone().foreground_color().foreground_color()),
        background: Some(iced::Background::Color(style.background_color().background_color())),
        ..Default::default()
    };

    let text = text(cell.content.to_string()).size(14).font(Font {
        family: iced::font::Family::Monospace,
        weight: iced::font::Weight::Normal, // TODO: actually handle font weights
        ..Font::default()
    });

    // TODO: Handle underline, strikethrough

    container(text).style(move |_| container_style).into()
}
