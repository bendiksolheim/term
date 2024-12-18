mod ansi_parser;
mod application;
mod config;
mod font;
mod structs;
mod term;
mod terminal;
mod window;

use crate::config::Config;
use application::Application;
use iced::{window::Id, Color, Size};
use window::WindowFocus;

fn main() -> iced::Result {
    let settings = settings();
    let config = Config::new();
    iced::daemon("Terminal", Application::update, Application::view)
        .style(|_state, _theme| iced::daemon::Appearance {
            background_color: Color::from_rgb(0.11764706, 0.11764706, 0.17647059),
            text_color: Color::from_rgb(0.0, 0.0, 0.0),
        })
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
    WindowFocus(WindowFocus),
}
