#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
    pub style: CursorStyle,
}

impl Cursor {
    pub fn set_position(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    pub fn left(&mut self, steps: usize, max: usize) {
        self.col = self.col.saturating_sub(steps).clamp(0, max);
    }

    pub fn right(&mut self, steps: usize, max: usize) {
        self.col = (self.col + steps).clamp(0, max);
    }

    pub fn up(&mut self, steps: usize, max: usize) {
        self.row = self.row.saturating_sub(steps).clamp(0, max);
    }

    pub fn down(&mut self, steps: usize, max: usize) {
        self.row = (self.row + steps).clamp(0, max);
    }

    pub fn set_style(&mut self, style: u8) {
        self.style = match style {
            0 | 1 => CursorStyle::BlinkingBlock,
            2 => CursorStyle::SteadyBlock,
            3 => CursorStyle::BlinkingUnderline,
            4 => CursorStyle::SteadyUnderline,
            5 => CursorStyle::BlinkingBar,
            6 => CursorStyle::SteadyBar,
            _ => {
                println!("Unknown cursor style {}", style);
                CursorStyle::default()
            }
        };
    }
}

pub enum Direction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum CursorStyle {
    BlinkingBlock,
    #[default]
    SteadyBlock,
    BlinkingUnderline,
    SteadyUnderline,
    BlinkingBar,
    SteadyBar,
}
