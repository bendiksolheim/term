#[derive(Default, Debug, Copy, Clone)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
}

impl Cursor {
    pub fn left(&mut self) {
        self.col = self.col - 1;
    }

    pub fn right(&mut self) {
        self.col = self.col + 1;
    }

    pub fn up(&mut self) {
        self.row = self.row - 1;
    }

    pub fn down(&mut self) {
        self.row = self.row + 1;
    }
}
