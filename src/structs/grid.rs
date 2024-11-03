use crate::structs::cursor::Cursor;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub rows: usize,
    pub cols: usize,
    data: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(rows * cols, data.len());
        Self { rows, cols, data }
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

    pub fn clear_selection(&mut self, selection: Selection) {
        match selection {
            Selection::Line(_cursor) => todo!(),
            Selection::FromStartOfLine(_cursor) => todo!(),
            Selection::ToEndOfLine(cursor) => {
                let from = cursor.row * self.cols + cursor.col;
                let to = (cursor.row + 1) * self.cols;
                self[from..to].fill(T::default());
            }
            Selection::ToEndOfDisplay(cursor) => {
                let from = cursor.row * self.cols + cursor.col;
                let to = self.data.len();
                self[from..to].fill(T::default());
            }
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        if rows < self.rows {
            // When removing rows, we remove from the top
            self.data.drain(0..self.cols);
            self.rows = rows;
        }

        if rows != self.rows || cols != self.cols {
            self.data.resize(rows * cols, T::default());
            self.rows = rows;
            self.cols = cols;
        }
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

impl<T> IndexMut<std::ops::Range<usize>> for Grid<T> {
    fn index_mut(&mut self, range: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.data[range]
    }
}

impl<T> Index<std::ops::Range<usize>> for Grid<T> {
    type Output = [T];

    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.data[range]
    }
}

pub enum Selection {
    Line(Cursor),
    FromStartOfLine(Cursor),
    ToEndOfLine(Cursor),
    ToEndOfDisplay(Cursor),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_grid_retains_correct_width_height() {
        let grid = Grid::new(10, 5, vec![1; 10 * 5]);
        assert_eq!(grid.cols, 5);
        assert_eq!(grid.rows, 10);
        assert_eq!(grid.data.len(), 5 * 10);
    }

    #[test]
    #[should_panic]
    fn creating_grid_with_wrong_content_length_throws() {
        Grid::new(10, 5, vec![1; 10 * 6]);
    }

    #[test]
    fn iterating_rows_should_create_correct_row_length() {
        let grid = Grid::new(3, 5, vec![1; 3 * 5]);
        let rows: Vec<&[i32]> = grid.iter_rows().collect();
        assert_eq!(rows.len(), 3);
        for row in rows {
            assert_eq!(row.len(), 5);
        }
    }

    #[test]
    fn shifting_row_preserves_size_and_adds_empty_row_last() {
        let mut grid = Grid::new(3, 5, vec![1; 3 * 5]);
        grid.shift_row();
        assert_eq!(grid.rows, 3);
        assert_eq!(grid.cols, 5);
        assert_eq!(grid.data.len(), 15);

        let rows: Vec<&[i32]> = grid.iter_rows().collect();
        assert_eq!(rows[0], vec![1, 1, 1, 1, 1]);
        assert_eq!(rows[1], vec![1, 1, 1, 1, 1]);
        assert_eq!(rows[2], vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn resizing_to_fewer_rows_works_correctly() {
        let mut grid = Grid::new(3, 5, vec![1; 3 * 5]);
        grid.resize(2, 5);

        assert_eq!(grid.rows, 2);
        assert_eq!(grid.cols, 5);
        assert_eq!(grid.data, vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn resizing_to_more_rows_works_correctly() {
        let mut grid = Grid::new(3, 2, vec![1; 3 * 2]);
        grid.resize(4, 2);

        assert_eq!(grid.rows, 4);
        assert_eq!(grid.cols, 2);
        assert_eq!(grid.data, vec![1, 1, 1, 1, 1, 1, 0, 0]);
    }

    #[test]
    fn resizing_to_more_cols_then_fewer_rows_works_correctly() {
        let mut grid = Grid::new(3, 2, vec![1; 3 * 2]);
        grid.resize(3, 3);
        assert_eq!(grid.data, vec![1, 1, 1, 1, 1, 1, 0, 0, 0]);
        grid.resize(2, 3);
        assert_eq!(grid.data, vec![1, 1, 1, 1, 1, 1]);
    }
}
