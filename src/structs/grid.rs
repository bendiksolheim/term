use crate::structs::cursor::Cursor;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Grid<T> {
    _rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(rows * cols, data.len());
        Self {
            _rows: rows,
            cols,
            data,
        }
    }

    // Iterate grid row by row
    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.cols)
    }

    // Removes first row and appends empty row last, in effect moving all lines up one row
    pub fn shift_row(&mut self) {
        let len = self.data.len();
        self.data.rotate_left(self.cols);
        self.data[len - self.cols..].fill(T::default());
    }
}

impl<T> Index<Cursor> for Grid<T> {
    type Output = T;

    fn index(&self, index: Cursor) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<T> IndexMut<Cursor> for Grid<T> {
    fn index_mut(&mut self, index: Cursor) -> &mut Self::Output {
        &mut self.data[index.row * self.cols + index.col]
    }
}
