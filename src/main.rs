mod structs {
    pub mod cell;
    pub mod cursor;
    pub mod grid;
    pub mod terminalsize;
}
mod terminal {
    pub mod colors;
    pub mod term;
}

use ansi_parser::AnsiParser;
use iced::{
    border,
    futures::{
        channel::mpsc::{self},
        SinkExt,
    },
    keyboard::{self, Key, Modifiers},
    widget::{container, text, Column, Row},
    Element, Font, Subscription, Task,
};
use structs::{
    cell::{Cell, CellStyle},
    cursor::Cursor,
    grid::Grid,
    terminalsize::TerminalSize,
};
use terminal::colors::TerminalColor;
use terminal::term::{Output, Winsize};

fn main() -> iced::Result {
    iced::application("Terminal", Terminalview::update, Terminalview::view)
        .decorations(false)
        .subscription(Terminalview::subscription)
        .run_with(Terminalview::new)
}

struct Terminalview {
    size: TerminalSize,
    cursor: Cursor,
    content: Grid<Cell>,
    current_cell_style: CellStyle,
    sender: Option<mpsc::Sender<terminal::term::TermMessage>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TerminalInput,
    Keyboard(Key, Modifiers),
    Term(terminal::term::Event),
}

impl Terminalview {
    fn new() -> (Self, Task<Message>) {
        let size = TerminalSize { cols: 80, rows: 24 };
        let content = Grid::new(size.rows, size.cols, vec![Cell::default(); size.rows * size.cols]);
        let model = Self {
            size,
            cursor: Cursor::default(),
            content,
            current_cell_style: CellStyle::default(),
            sender: None,
        };

        (model, Task::none())
    }

    fn view(&self) -> Element<'_, Message> {
        container(Column::with_children(
            self.content
                .iter_rows()
                .map(|row| {
                    container(Row::with_children(
                        row.iter()
                            .map(|cell| {
                                text(cell.content.to_string())
                                    .font(Font::MONOSPACE)
                                    .size(14)
                                    .color(cell.style.foreground.foreground_color())
                                    .into()
                            })
                            .collect::<Vec<_>>(),
                    ))
                    .style(|_| {
                        container::Style::default()
                            .border(border::color(TerminalColor::Green.foreground_color()).width(1))
                    })
                    .into()
                })
                .collect::<Vec<_>>(),
        ))
        .style(|_| container::Style::default().border(border::color(TerminalColor::Red.foreground_color()).width(2)))
        .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Keyboard(k, _) => {
                if let Some(sender) = self.sender.clone() {
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
            Message::Term(term_event) => match term_event {
                terminal::term::Event::Ready(sender) => {
                    self.sender = Some(sender);
                    Task::none()
                }
                terminal::term::Event::Output(s) => {
                    match s {
                        Output::Text(s) => {
                            self.handle_ansi(&s);
                        }

                        Output::NewLine => {
                            self.cursor.down();
                        }

                        Output::CarriageReturn => {
                            self.cursor.col = 0;
                        }

                        Output::Backspace => {
                            self.cursor.left();
                            self.content[self.cursor] = Cell::default();
                        }
                    }
                    Task::none()
                }
            },
        }
    }

    fn handle_ansi(&mut self, ansi_text: &str) {
        let parsed = ansi_text.ansi_parse();
        for block in parsed.into_iter() {
            match block {
                ansi_parser::Output::TextBlock(text) => text.chars().for_each(|c| {
                    self.content[self.cursor].content = c;
                    self.content[self.cursor].style = self.current_cell_style;
                    self.cursor.right();
                }),
                ansi_parser::Output::Escape(code) => match code {
                    ansi_parser::AnsiSequence::SetGraphicsMode(color) => {
                        if color.len() == 1 {
                            let term_color = TerminalColor::parse_ansi(color[0]);
                            self.current_cell_style.foreground = term_color;
                        }
                    }

                    _ => {
                        println!("{:?}", code);
                    }
                },
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keypress = keyboard::on_key_press(|key, modifier| Some(Message::Keyboard(key, modifier)));
        let winsize = create_winsize(self.size);
        let term_sub = Subscription::run_with_id(12345, terminal::term::Term::spawn(winsize)).map(Message::Term);
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
