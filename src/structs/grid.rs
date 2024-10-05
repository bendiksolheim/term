use crate::structs::cursor::Cursor;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T> Grid<T> {
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(rows * cols, data.len());
        Self { rows, cols, data }
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.cols)
    }
}
impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0 * self.cols + index.1]
    }
}

impl<T> Index<Cursor> for Grid<T> {
    type Output = T;

    fn index(&self, index: Cursor) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 * self.cols + index.1]
    }
}

impl<T> IndexMut<Cursor> for Grid<T> {
    fn index_mut(&mut self, index: Cursor) -> &mut Self::Output {
        &mut self.data[index.row * self.cols + index.col]
    }
}
