use iced::{Padding, Size};

use crate::font::Font;

#[derive(Debug, Clone)]
pub struct Config {
    pub font_size: f32,
    pub window_size: Size,
    pub window_padding: Padding,
    pub cell_size: Size,
}

impl Config {
    pub fn new() -> Self {
        let font_size = 14.0;
        let font_measure = Font::measure_text("Iosevka", 'M', font_size);
        let cell_size = Size {
            width: font_measure.width * CHARACTER_WIDTH_FACTOR,
            height: font_measure.height * CHARACTER_HEIGHT_FACTOR,
        };
        Self {
            font_size,
            window_size: Size {
                width: 1024.0,
                height: 726.0,
            },
            window_padding: Padding {
                top: 25.0,
                left: 5.0,
                bottom: 5.0,
                right: 5.0,
            },
            cell_size,
        }
    }
}

static CHARACTER_WIDTH_FACTOR: f32 = 1.25; // Found by trial and error
static CHARACTER_HEIGHT_FACTOR: f32 = 1.3; // Found by trial and error
