use iced::{
    window::{settings::PlatformSpecific, Id, Settings},
    Padding, Size, Task,
};

pub struct Window {
    _id: Id,
    pub size: Size,
    pub padding: Padding,
}

impl Window {
    pub fn main_window(config: WindowConfig) -> (Self, Task<Id>) {
        let (id, task) = iced::window::open(terminal_window_settings(config.size));
        (
            Self {
                _id: id,
                size: config.size,
                padding: config.padding,
            },
            task,
        )
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
    }

    pub fn content_width(&self) -> f32 {
        self.size.width - self.padding.horizontal()
    }

    pub fn content_height(&self) -> f32 {
        self.size.height - self.padding.vertical()
    }
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub size: Size,
    pub padding: Padding,
}

#[derive(Debug, Clone)]
pub enum WindowFocus {
    Focus,
    Unfocus,
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
