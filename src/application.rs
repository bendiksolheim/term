use iced::{
    keyboard::{self, key::Named, Key},
    widget::{container, text, Column, Container, Row},
    window::Id,
    Element, Subscription, Task,
};

use crate::{
    config::Config,
    structs::{cell::Cell, cursor::Cursor, terminalsize::TerminalSize},
    term::{colors::TerminalColor, term},
    terminal::Terminal,
    window::{Window, WindowFocus},
    Message,
};

pub struct Application {
    terminal: Terminal,
    config: Config,
    window: Window,
}

impl Application {
    pub fn new(config: Config) -> (Self, Task<Message>) {
        let (window, window_task) = Window::main_window(config.window_config.clone());
        let cols = (window.content_width() / config.cell_size.width) as usize;
        let rows = (window.content_height() / config.cell_size.height) as usize;
        let size = TerminalSize { cols, rows };

        (
            Self {
                terminal: Terminal::new(size),
                config,
                window,
            },
            window_task.map(|id| Message::WindowCreated(id)),
        )
    }

    pub fn view(&self, _window: Id) -> Element<'_, Message> {
        let buffer = self.terminal.buffer();
        Column::with_children(
            buffer
                .iter_rows()
                .enumerate()
                .map(|(y, row)| {
                    Row::with_children(
                        row.iter()
                            .enumerate()
                            .map(|(x, cell)| {
                                cell_view(&buffer.cursor, x, y, cell, self.config.font_size)
                                    .width(self.config.cell_size.width)
                                    .height(self.config.cell_size.height)
                                    .into()
                            })
                            .collect::<Vec<_>>(),
                    )
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .padding(self.window.padding)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyboardBytes(bytes) => self.terminal.send(term::TermMessage::Bytes(bytes)),
            Message::TerminalInput => Task::none(),
            Message::TerminalOutput(term_event) => self.terminal.parse(term_event),
            Message::WindowCreated(_id) => Task::none(),
            Message::WindowResized(size) => {
                let current_cols = (self.window.content_width() / self.config.cell_size.width) as usize;
                let current_rows = (self.window.content_height() / self.config.cell_size.height) as usize;
                let new_cols = ((size.width - self.window.padding.horizontal()) / self.config.cell_size.width) as usize;
                let new_rows = ((size.height - self.window.padding.vertical()) / self.config.cell_size.height) as usize;
                if new_cols != current_cols || new_rows != current_rows {
                    let new_size = TerminalSize::new(new_cols, new_rows);
                    self.window.resize(size);
                    self.terminal.resize(new_size)
                } else {
                    Task::none()
                }
            }
            Message::WindowFocus(focus) => match focus {
                WindowFocus::Focus => self.terminal.focus(),
                WindowFocus::Unfocus => self.terminal.unfocus(),
            },
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let tmp = iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key,
                    modified_key: _,
                    physical_key: _,
                    location: _,
                    modifiers: _,
                    text,
                } => {
                    if let Some(char) = text {
                        Some(Message::KeyboardBytes(char.as_bytes().to_vec()))
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
            iced::Event::Window(event) => match event {
                iced::window::Event::Resized(size) => Some(Message::WindowResized(size)),
                iced::window::Event::Focused => Some(Message::WindowFocus(WindowFocus::Focus)),
                iced::window::Event::Unfocused => Some(Message::WindowFocus(WindowFocus::Unfocus)),
                _ => None,
            },
            iced::Event::Touch(_event) => None,
        });
        let term_sub =
            Subscription::run_with_id(12345, term::Term::spawn(self.terminal.winsize())).map(Message::TerminalOutput);
        iced::Subscription::batch([tmp, term_sub])
    }
}

fn cell_view<'a>(cursor: &Cursor, x: usize, y: usize, cell: &Cell, font_size: f32) -> Container<'a, Message> {
    let mut container_style = container::Style {
        // TODO: Do I really need to clone here?
        text_color: Some(cell.style.clone().foreground_color().foreground_color()),
        background: Some(iced::Background::Color(
            cell.style.clone().background_color().background_color(),
        )),
        border: iced::Border::default()
            .width(0.0)
            .color(TerminalColor::Cyan.foreground_color()),
        ..Default::default()
    };

    if cursor.col == x && cursor.row == y {
        use crate::structs::cursor::CursorStyle::*;
        match cursor.style {
            BlinkingBlock | SteadyBlock => {
                container_style.text_color = Some(TerminalColor::Black.foreground_color());
                container_style.background = Some(iced::Background::Color(TerminalColor::White.background_color()));
            }
            BlinkingUnderline | SteadyUnderline => {
                let gradient = underline_cursor();
                container_style.background = Some(iced::Background::Gradient(iced::Gradient::Linear(gradient)));
            }
            BlinkingBar | SteadyBar => {
                let gradient = bar_cursor();
                container_style.background = Some(iced::Background::Gradient(iced::Gradient::Linear(gradient)));
            }
        };
    }
    let text = text(cell.content.to_string()).size(font_size);

    // TODO: Handle underline, strikethrough

    container(text).style(move |_| {
        container_style.border(
            iced::Border::default()
                .color(iced::Color::from_rgb(0.0, 1.0, 0.0))
                .width(0.5),
        )
    })
}

// We fake underline cursor with a gradient to draw a line at the bottom of a cell
fn underline_cursor() -> iced::gradient::Linear {
    iced::gradient::Linear::new(0.0)
        .add_stop(0.0, TerminalColor::White.background_color())
        .add_stop(0.1, TerminalColor::White.background_color())
        .add_stop(0.11, TerminalColor::Black.background_color())
        .add_stop(1.0, TerminalColor::Black.background_color())
}

// We fake a bar cursor with a gradient to draw a line on the left side of a cell
fn bar_cursor() -> iced::gradient::Linear {
    iced::gradient::Linear::new(1.57079633) // PI / 2
        .add_stop(0.0, TerminalColor::White.background_color())
        .add_stop(0.1, TerminalColor::White.background_color())
        .add_stop(0.1001, TerminalColor::Black.background_color())
        .add_stop(1.0, TerminalColor::Black.background_color())
}
