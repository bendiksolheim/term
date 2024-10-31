mod config;
mod font;

mod structs {
    pub mod cell;
    pub mod cursor;
    pub mod grid;
    pub mod terminalsize;
}
mod terminal {
    pub mod colors;
    pub mod font;
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
    keyboard::{self, key::Named, Key},
    widget::{container, text, Column, Row},
    window::{settings::PlatformSpecific, Id, Settings},
    Element, Font, Padding, Pixels, Size, Subscription, Task,
};
use structs::{
    cell::{Cell, CellStyle},
    cursor::Cursor,
    grid::{Grid, Selection},
    terminalsize::TerminalSize,
};
use terminal::term::Winsize;
use terminal::{colors::TerminalColor, terminal_output::TerminalOutput};

use crate::config::Config;

fn main() -> iced::Result {
    let settings = settings();
    let config = Config::new();
    iced::daemon("Terminal", Terminalview::update, Terminalview::view)
        .settings(settings)
        .subscription(Terminalview::subscription)
        .run_with(move || Terminalview::new(config))
}

fn settings() -> iced::Settings {
    iced::Settings {
        id: None,
        fonts: vec![],
        default_font: Font::with_name("Iosevka"),
        default_text_size: 14.0.into(),
        antialiasing: false,
    }
}

pub struct Terminalview {
    application_mode: bool, // Changes how cursor keys are coded
    newline_mode: bool,     // Interprets \n as NL LF instead of just NL
    size: TerminalSize,
    cursor: Cursor,
    cursor_visible: bool,
    saved_cursor_position: Option<Cursor>,
    content: Grid<Cell>,
    current_cell_style: CellStyle,
    sender: Option<mpsc::Sender<terminal::term::TermMessage>>,
    windows: BTreeMap<Id, WindowType>,
    debug: DebugState<Message>,
    config: Config,
}

#[derive(Debug, Clone)]
pub enum WindowType {
    TerminalWindow,
    DebugWindow,
}

#[derive(Debug, Clone)]
pub enum Message {
    TerminalInput,
    KeyboardBytes(Vec<u8>),
    TerminalOutput(terminal::term::Event),
    WindowCreated(Id, WindowType),
    ShowDebugWindow,
    ShowMessage(Box<Message>),
}

impl Message {
    fn name(&self) -> &str {
        match self {
            Message::TerminalInput => "TerminalInput",
            Message::KeyboardBytes(_bytes) => "KeyboardBytes(bytes)",
            Message::TerminalOutput(_event) => "TerminalOutput(event)",
            Message::WindowCreated(_id, _window_type) => "WindowCreated(id, window_type)",
            Message::ShowDebugWindow => "ToggleDebugWindow",
            Message::ShowMessage(_message) => "ShowMessage(message)",
        }
    }
}

static CHARACTER_WIDTH_FACTOR: f32 = 1.25; // Found by trial and error
static CHARACTER_HEIGHT_FACTOR: f32 = 1.3; // Found by trial and error

impl Terminalview {
    fn new(config: Config) -> (Self, Task<Message>) {
        let font_measure = measure_text("Iosevka", 'M', config.font_size);
        let cell_size = Size {
            width: font_measure.width * CHARACTER_WIDTH_FACTOR,
            height: font_measure.height * CHARACTER_HEIGHT_FACTOR,
        };
        let cols = ((config.window_size.width - (config.window_padding.left + config.window_padding.right))
            / cell_size.width) as usize;
        let rows = ((config.window_size.height - (config.window_padding.top + config.window_padding.bottom))
            / cell_size.height) as usize;
        // println!("Cols: {:?}, rows: {:?}", cols, rows);
        let size = TerminalSize { cols, rows };
        let (id, terminal_window) = iced::window::open(terminal_window_settings(config.window_size));
        let mut windows = BTreeMap::new();
        windows.insert(id, WindowType::TerminalWindow); // Needed to render instantly, otherwise we get an initial render delay
        let content = Grid::new(size.rows, size.cols, vec![Cell::default(); size.rows * size.cols]);
        let model = Self {
            application_mode: false,
            newline_mode: false,
            size,
            cursor: Cursor::default(),
            cursor_visible: true,
            saved_cursor_position: None,
            content,
            current_cell_style: CellStyle::default(),
            sender: None,
            windows,
            debug: DebugState::default(),
            config,
        };

        (
            model,
            terminal_window.map(|id| Message::WindowCreated(id, WindowType::TerminalWindow)),
        )
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
        .padding(self.config.window_padding)
        .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.debug.messages.push(message.clone());
        match message {
            Message::KeyboardBytes(bytes) => {
                if let Some(sender) = self.sender.clone() {
                    let f = async move {
                        let mut sender = sender;
                        sender
                            .send(terminal::term::TermMessage::Bytes(bytes))
                            .await
                            .expect("Could not send TermMessage:Bytes");
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
            },
            Message::WindowCreated(id, window_type) => {
                self.windows.insert(id, window_type);
                Task::none()
            }
            Message::ShowDebugWindow => {
                let (_id, debug_window) = iced::window::open(debug_window_settings());
                debug_window.map(|id| Message::WindowCreated(id, WindowType::DebugWindow))
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

    fn subscription(&self) -> Subscription<Message> {
        let tmp = iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key,
                    modified_key: _,
                    physical_key: _,
                    location: _,
                    modifiers,
                    text,
                } => {
                    if let Some(char) = text {
                        if char == "\r" && modifiers.command() {
                            Some(Message::ShowDebugWindow)
                        } else {
                            println!("Char: {:?}", char);
                            Some(Message::KeyboardBytes(char.as_bytes().to_vec()))
                        }
                    } else if let Key::Named(k) = key {
                        match k {
                            Named::ArrowUp => Some(Message::KeyboardBytes("\x1b[A".into())),
                            Named::ArrowDown => Some(Message::KeyboardBytes("\x1b[B".into())),
                            Named::ArrowRight => Some(Message::KeyboardBytes("\x1b[C".into())),
                            Named::ArrowLeft => Some(Message::KeyboardBytes("\x1b[D".into())),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }

                // We dont care about KeyReleased and ModifiersChanged
                _ => None,
            },
            iced::Event::Mouse(_event) => None,
            iced::Event::Window(_event) => None,
            iced::Event::Touch(_event) => None,
        });
        let winsize = create_winsize(self.size);
        let term_sub =
            Subscription::run_with_id(12345, terminal::term::Term::spawn(winsize)).map(Message::TerminalOutput);
        iced::Subscription::batch([tmp, term_sub])
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

fn terminal_window_settings(size: Size) -> Settings {
    Settings {
        decorations: true,
        platform_specific: PlatformSpecific {
            title_hidden: true,
            titlebar_transparent: true,
            fullsize_content_view: true,
        },
        size,
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
        border: iced::Border::default()
            .width(Pixels(0.5))
            .color(TerminalColor::Cyan.foreground_color()),
        ..Default::default()
    };

    let text = text(cell.content.to_string())
        // .width(5.6000004 * 1.25)
        // .height(14.0 * 1.25)
        .size(14);
    // TODO: Handle underline, strikethrough

    container(text).style(move |_| container_style).into()
}

fn measure_text(font: &str, text: char, font_size: f32) -> Size {
    use rusttype::{Font, Scale};
    let font_data = load_font(font);
    let font = Font::try_from_bytes(&font_data).unwrap();

    let scale = Scale::uniform(font_size);
    let v_metrics = font.v_metrics(scale);
    let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

    let glyph = font.glyph(text).scaled(scale);
    let h_metrics = glyph.h_metrics();
    println!("left_side_bearing: {:?}", h_metrics.left_side_bearing);
    let width = h_metrics.advance_width;

    Size {
        width: width,
        height: line_height,
    }
}

fn load_font(font: &str) -> Vec<u8> {
    use font_loader::system_fonts;
    let property = font_loader::system_fonts::FontPropertyBuilder::new()
        .family(font)
        .build();

    let (font_data, _) = system_fonts::get(&property).unwrap();

    font_data
}
