use iced::Size;

pub struct Font;

impl Font {
    pub fn measure_text(font: &str, text: char, font_size: f32) -> Size {
        use rusttype::{Font, Scale};
        let font_data = Self::load_font(font);
        let font = Font::try_from_bytes(&font_data).unwrap();

        let scale = Scale::uniform(font_size);
        let v_metrics = font.v_metrics(scale);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        let glyph = font.glyph(text).scaled(scale);
        let h_metrics = glyph.h_metrics();
        println!("left_side_bearing: {:?}", h_metrics.left_side_bearing);
        let width = h_metrics.advance_width;

        Size {
            width: width,
            height: line_height,
        }
    }

    fn load_font(font: &str) -> Vec<u8> {
        use font_loader::system_fonts;
        let property = font_loader::system_fonts::FontPropertyBuilder::new()
            .family(font)
            .build();

        let (font_data, _) = system_fonts::get(&property).unwrap();

        font_data
    }
}
