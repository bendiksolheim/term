use iced::{
    keyboard::{self, key::Named, Key},
    widget::{container, text, Column, Row},
    window::Id,
    Element, Subscription, Task,
};

use crate::{
    config::Config,
    structs::{cell::Cell, cursor::Cursor, terminalsize::TerminalSize},
    term::{colors::TerminalColor, term},
    terminal::Terminal,
    window::Window,
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
        Column::with_children(
            self.terminal
                .content
                .iter_rows()
                .enumerate()
                .map(|(y, row)| {
                    Row::with_children(
                        row.iter()
                            .enumerate()
                            .map(|(x, cell)| cell_view(&self.terminal.cursor, x, y, cell, self.config.font_size))
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
                let cols_changed = (size.width - self.window.size.width).abs() > self.config.cell_size.width;
                let rows_changed = (size.height - self.window.size.height).abs() > self.config.cell_size.height;
                if cols_changed || rows_changed {
                    let cols = (self.window.content_width() / self.config.cell_size.width) as usize;
                    let rows = (self.window.content_height() / self.config.cell_size.height) as usize;
                    let new_size = TerminalSize { cols, rows };
                    self.terminal.resize(new_size)
                } else {
                    Task::none()
                }
            }
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
                _ => None,
            },
            iced::Event::Touch(_event) => None,
        });
        let term_sub =
            Subscription::run_with_id(12345, term::Term::spawn(self.terminal.winsize())).map(Message::TerminalOutput);
        iced::Subscription::batch([tmp, term_sub])
    }
}

fn cell_view<'a>(cursor: &Cursor, x: usize, y: usize, cell: &Cell, font_size: f32) -> Element<'a, Message> {
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
            .width(0.0)
            .color(TerminalColor::Cyan.foreground_color()),
        ..Default::default()
    };

    let text = text(cell.content.to_string()).size(font_size);

    // TODO: Handle underline, strikethrough

    container(text).style(move |_| container_style).into()
}
