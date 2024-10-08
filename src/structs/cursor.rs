#[derive(Default, Debug, Copy, Clone)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
}

impl Cursor {
    pub fn left(&mut self, steps: usize) {
        self.col = self.col - steps;
    }

    pub fn right(&mut self, steps: usize) {
        self.col = self.col + steps;
    }

    pub fn up(&mut self) {
        self.row = self.row - 1;
    }

    pub fn down(&mut self) {
        self.row = self.row + 1;
    }
}
