mod ansi_parser;
mod application;
mod config;
mod font;
mod terminal;
mod window;

mod structs {
    pub mod buffer;
    pub mod cell;
    pub mod cursor;
    pub mod terminalsize;
}
mod term {
    pub mod colors;
    pub mod font;
    pub mod graphics;
    pub mod pty_reader;
    pub mod term;
    pub mod terminal_output;
}

use crate::config::Config;
use application::Application;
use iced::{window::Id, Size};

fn main() -> iced::Result {
    let settings = settings();
    let config = Config::new();
    iced::daemon("Terminal", Application::update, Application::view)
        .settings(settings)
        .subscription(Application::subscription)
        .run_with(move || Application::new(config))
}

fn settings() -> iced::Settings {
    iced::Settings {
        id: None,
        fonts: vec![],
        default_font: iced::Font::with_name("Iosevka"),
        default_text_size: 14.0.into(),
        antialiasing: false,
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TerminalInput,
    KeyboardBytes(Vec<u8>),
    TerminalOutput(term::term::Event),
    WindowCreated(Id),
    WindowResized(Size),
}
