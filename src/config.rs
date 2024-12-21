use iced::{Padding, Size};

use crate::{font::Font, window::WindowConfig};

#[derive(Debug, Clone)]
pub struct Config {
    pub font_size: f32,
    pub window_config: WindowConfig,
    pub cell_size: Size,
}

impl Config {
    pub fn new(font: &Font) -> Self {
        Self {
            font_size: font.size,
            window_config: WindowConfig {
                size: Size {
                    width: 1024.0,
                    height: 726.0,
                },
                padding: Padding {
                    top: 25.0,
                    left: 5.0,
                    bottom: 5.0,
                    right: 5.0,
                },
            },
            cell_size: font.measure_glyph("M"),
        }
    }
}
