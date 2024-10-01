mod term;

use iced::{
    futures::{
        channel::mpsc::{self},
        SinkExt,
    },
    keyboard::{self, Key, Modifiers},
    widget::{column, text, Column},
    Color, Font, Subscription, Task,
};
use rustix_openpty::rustix::termios::Winsize;
use term::Output;

fn main() -> iced::Result {
    iced::application("Terminal", Terminalview::update, Terminalview::view)
        .decorations(false)
        .subscription(Terminalview::subscription)
        .run_with(Terminalview::new)
}

struct Terminalview {
    width: u16,
    height: u16,
    content: String,
    sender: Option<mpsc::Sender<term::TermMessage>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TerminalOutput(String),
    TerminalInput,
    Keyboard(Key, Modifiers),
    Term(term::Event),
}

impl Terminalview {
    fn new() -> (Self, Task<Message>) {
        let model = Self {
            width: 80,
            height: 24,
            content: String::new(),
            sender: None,
        };

        (model, Task::none())
    }

    fn view(&self) -> Column<Message> {
        column![text(self.content.clone())
            .font(Font::MONOSPACE)
            .color(Color::from_rgb8(0xff, 0xff, 0xff))
            .size(14),]
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TerminalOutput(s) => {
                self.content.push_str(&s);
                Task::none()
            }
            Message::Keyboard(k, _) => {
                if let Some(sender) = self.sender.clone() {
                    let f = async move {
                        let mut sender = sender;
                        sender
                            .send(term::TermMessage::Input(k))
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
                term::Event::Ready(sender) => {
                    self.sender = Some(sender);
                    Task::none()
                }
                term::Event::Output(s) => {
                    match s {
                        Output::Text(s) => {
                            self.content.push_str(&s);
                        }
                        Output::Newline => {
                            self.content.push('\n');
                        }
                        Output::Backspace => {
                            self.content.remove(1);
                        }
                    }
                    Task::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keypress = keyboard::on_key_press(|key, modifier| Some(Message::Keyboard(key, modifier)));
        let winsize = create_winsize(self.width, self.height);
        let term_sub = Subscription::run_with_id(12345, term::Term::spawn(winsize)).map(Message::Term);
        iced::Subscription::batch([term_sub, keypress])
    }
}

fn create_winsize(width: u16, height: u16) -> Winsize {
    Winsize {
        ws_col: width,
        ws_row: height,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}
