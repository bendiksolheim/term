use iced::Color;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum TerminalColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
    EightBit(u8),
}

static COLOR_MAP: Lazy<HashMap<u8, Color>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                let code = 16 + 36 * r + 6 * g + b;
                let red = scale_to_256(r);
                let green = scale_to_256(g);
                let blue = scale_to_256(b);
                m.insert(code, Color::from_rgb8(red, green, blue));
            }
        }
    }

    for g in 232..=255 {
        let gray = (g - 232) * 10 + 8;
        m.insert(g, Color::from_rgb8(gray, gray, gray));
    }
    m
});

impl Default for TerminalColor {
    fn default() -> Self {
        TerminalColor::Default
    }
}

/*
"brightBlack": "#5B6078",
  "brightRed": "#ED8796",
  "brightGreen": "#A6DA95",
  "brightYellow": "#EED49F",
  "brightBlue": "#8AADF4",
  "brightPurple": "#F5BDE6",
  "brightCyan": "#8BD5CA",
  "brightWhite": "#A5ADCB"
*/

impl TerminalColor {
    pub fn foreground_color(&self) -> Color {
        match self {
            TerminalColor::Black => Color::from_rgb(0.286, 0.302, 0.392),
            TerminalColor::Red => Color::from_rgb(0.929, 0.529, 0.588),
            TerminalColor::Green => Color::from_rgb(0.651, 0.855, 0.584),
            TerminalColor::Yellow => Color::from_rgb(0.933, 0.831, 0.624),
            TerminalColor::Blue => Color::from_rgb(0.541, 0.678, 0.957),
            TerminalColor::Magenta => Color::from_rgb(0.961, 0.741, 0.902),
            TerminalColor::Cyan => Color::from_rgb(0.545, 0.835, 0.792),
            TerminalColor::White => Color::from_rgb(0.722, 0.753, 0.878),
            TerminalColor::Default => Color::from_rgb(1.0, 1.0, 1.0),
            TerminalColor::EightBit(n) => parse_eight_bit_color(n),
        }
    }

    pub fn background_color(&self) -> Color {
        match self {
            TerminalColor::Black => Color::from_rgb(0.286, 0.302, 0.392),
            TerminalColor::Red => Color::from_rgb(0.929, 0.529, 0.588),
            TerminalColor::Green => Color::from_rgb(0.651, 0.855, 0.584),
            TerminalColor::Yellow => Color::from_rgb(0.933, 0.831, 0.624),
            TerminalColor::Blue => Color::from_rgb(0.541, 0.678, 0.957),
            TerminalColor::Magenta => Color::from_rgb(0.961, 0.741, 0.902),
            TerminalColor::Cyan => Color::from_rgb(0.545, 0.835, 0.792),
            TerminalColor::White => Color::from_rgb(0.722, 0.753, 0.878),
            TerminalColor::Default => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            TerminalColor::EightBit(n) => parse_eight_bit_color(n),
        }
    }
}

fn parse_eight_bit_color(n: &u8) -> Color {
    match n {
        // Regular color scheme
        0 => Color::from_rgb(0.286, 0.302, 0.392),
        1 => Color::from_rgb(0.929, 0.529, 0.588),
        2 => Color::from_rgb(0.651, 0.855, 0.584),
        3 => Color::from_rgb(0.933, 0.831, 0.624),
        4 => Color::from_rgb(0.541, 0.678, 0.957),
        5 => Color::from_rgb(0.961, 0.741, 0.902),
        6 => Color::from_rgb(0.545, 0.835, 0.792),
        7 => Color::from_rgb(0.722, 0.753, 0.878),

        // Bright colors – just using regular colors for now
        8 => Color::from_rgb(0.286, 0.302, 0.392),
        9 => Color::from_rgb(0.929, 0.529, 0.588),
        10 => Color::from_rgb(0.651, 0.855, 0.584),
        11 => Color::from_rgb(0.933, 0.831, 0.624),
        12 => Color::from_rgb(0.541, 0.678, 0.957),
        13 => Color::from_rgb(0.961, 0.741, 0.902),
        14 => Color::from_rgb(0.545, 0.835, 0.792),
        15 => Color::from_rgb(0.722, 0.753, 0.878),

        16..=255 => COLOR_MAP
            .get(n)
            .expect(format!("Expected SGR 8 bit color {} to be precalculated", n).as_str())
            .clone(),
    }
}

fn scale_to_256(n: u8) -> u8 {
    if n == 0 {
        0
    } else {
        n * 40 + 55
    }
}
