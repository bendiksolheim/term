use iced::widget::button;
use iced::widget::container;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::Column;
use iced::Border;
use iced::Element;

use crate::terminal::colors::TerminalColor;
use crate::Message;
use crate::Terminalview;

pub struct DebugState<M> {
    pub messages: Vec<M>,
    pub selected: Option<M>,
}

impl<M> Default for DebugState<M> {
    fn default() -> Self {
        Self {
            messages: vec![],
            selected: None,
        }
    }
}

impl Terminalview {
    pub fn debug_view(&self) -> Element<'_, Message> {
        let list = scrollable(
            container(Column::with_children(self.debug.messages.iter().map(|message| {
                button(message.name())
                    .on_press(Message::ShowMessage(Box::new(message.clone())))
                    .into()
            })))
            .style(|_| {
                container::Style::default().border(
                    Border::default()
                        .color(TerminalColor::White.foreground_color())
                        .width(2),
                )
            }),
        )
        .into();

        if let Some(details) = &self.debug.selected {
            let details_view = container(text(format!("{:?}", details)));
            row![list, details_view].into()
        } else {
            list
        }
    }
}
