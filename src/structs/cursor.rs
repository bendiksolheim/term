#[derive(Default, Debug, Copy, Clone)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
}

impl Cursor {
    pub fn set_position(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    pub fn left(&mut self, steps: usize) {
        self.col = self.col - steps;
    }

    pub fn right(&mut self, steps: usize) {
        self.col = self.col + steps;
    }

    pub fn up(&mut self, steps: usize) {
        self.row = self.row - steps;
    }

    pub fn down(&mut self, steps: usize) {
        self.row = self.row + steps;
    }
}
