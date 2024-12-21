use iced::{widget::text::LineHeight, Size};

pub struct Font {
    pub name: &'static str,
    pub size: f32,
}

impl Font {
    pub fn new(name: &'static str, size: f32) -> Self {
        Self { name, size }
    }

    pub fn measure_glyph(&self, char: &str) -> Size {
        use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};
        let line_height_scale = LineHeight::default();
        let line_height = line_height_scale.to_absolute(self.size.into()).0;

        let mut font_system = FontSystem::new();
        let mut buffer = Buffer::new_empty(Metrics {
            font_size: self.size,
            line_height,
        });
        let font_attributes = Attrs::new().family(Family::Name(self.name));
        buffer.set_text(&mut font_system, char, font_attributes, Shaping::Advanced);

        let width = buffer.layout_runs().fold(0.0, |width, run| run.line_w.max(width));

        Size {
            width,
            height: buffer.metrics().line_height,
        }
    }
}
