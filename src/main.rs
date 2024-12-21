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
use font::Font;
use iced::{window::Id, Color, Size};
use window::WindowFocus;

fn main() -> iced::Result {
    let font = Font::new("Iosevka", 14.0);
    let settings = settings(&font);
    let config = Config::new(&font);
    iced::daemon("Terminal", Application::update, Application::view)
        .style(|_state, _theme| iced::daemon::Appearance {
            background_color: Color::from_rgb(0.11764706, 0.11764706, 0.17647059),
            text_color: Color::from_rgb(0.0, 0.0, 0.0),
        })
        .settings(settings)
        .subscription(Application::subscription)
        .run_with(move || Application::new(config))
}

fn settings(font: &Font) -> iced::Settings {
    iced::Settings {
        id: None,
        fonts: vec![],
        default_font: iced::Font::with_name(font.name),
        default_text_size: font.size.into(),
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
