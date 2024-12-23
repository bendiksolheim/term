use crate::structs::cursor::Cursor;
use std::ops::{Index, IndexMut};

use super::{
    cell::{Cell, CellStyle},
    cursor::Direction,
};

#[derive(Debug, Clone)]
pub struct Buffer<T> {
    pub rows: usize,
    pub cols: usize,
    data: Vec<T>,
    top: usize,
    bottom: usize,
    pub cursor: Cursor,
    saved_cursor: Option<Cursor>,
}

impl<T: Clone + Default + Copy> Buffer<T> {
    pub fn new(rows: usize, cols: usize, data: Vec<T>) -> Self {
        assert_eq!(rows * cols, data.len());
        Self {
            rows,
            cols,
            data,
            top: 0,
            bottom: rows - 1,
            cursor: Cursor::default(),
            saved_cursor: None,
        }
    }

    pub fn get(&mut self, cursor: Cursor) -> Option<&mut T> {
        let index = cursor.row * self.cols + cursor.col;
        self.data.get_mut(index)
    }

    // Iterate grid row by row
    pub fn iter_rows(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.cols)
    }

    // Appends empty row last and removes first row inside scroll area
    pub fn shift_row(&mut self) {
        let from = self.top * self.cols;
        let to = (self.bottom + 1) * self.cols;
        self.data.copy_within(from + self.cols..to, from);
        self.data[(to - self.cols)..to].fill(T::default());
    }

    // Prepends empty row first and removes last row inside scroll area
    pub fn unshift_row(&mut self) {
        if self.cursor.row == self.top {
            let from = self.top * self.cols; // 0
            let to = self.bottom * self.cols; // 8
            self.data.copy_within(from..to, from + self.cols);
            self.data[from..(from + self.cols)].fill(T::default());
        } else {
            self.cursor.up(1, self.rows - 1);
        }
    }

    pub fn clear_selection(&mut self, selection: Selection) {
        match selection {
            Selection::Line => {
                let from = self.cursor.row * self.cols;
                let to = (self.cursor.row + 1) * self.cols;
                self[from..to].fill(T::default());
            }
            Selection::FromStartOfLine => {
                let from = self.cursor.row * self.cols;
                let to = self.cursor.row * self.cols + self.cursor.col;
                self[from..to].fill(T::default());
            }
            Selection::ToEndOfLine => {
                let from = self.cursor.row * self.cols + self.cursor.col;
                let to = (self.cursor.row + 1) * self.cols;
                self[from..to].fill(T::default());
            }
            Selection::ToEndOfDisplay => {
                let from = self.cursor.row * self.cols + self.cursor.col;
                let to = self.data.len();
                self[from..to].fill(T::default());
            }
            Selection::Characters(n) => {
                let from = self.cursor.row * self.cols + self.cursor.col;
                let to = from + n as usize;
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

        if self.cursor.col >= self.cols {
            self.cursor.up(self.cursor.col - self.cols, self.rows);
        }

        if self.cursor.row >= self.rows {
            self.cursor.left(self.cursor.row - self.rows, self.cols);
        }
    }

    pub fn newline(&mut self, newline_mode: bool) {
        if self.cursor.row == self.rows - 1 {
            self.shift_row();
        } else {
            self.cursor.down(1, self.rows);
        }

        // If terminal is in newline mode, cursor is also moved to start of line
        if newline_mode {
            self.cursor.col = 0;
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }

    pub fn backspace(&mut self) {
        self.cursor.left(1, self.cols);
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor.clone());
    }

    pub fn restore_cursor(&mut self) {
        if let Some(cursor) = self.saved_cursor {
            self.cursor = cursor;
            self.saved_cursor = None;
        }
    }

    pub fn set_top_bottom(&mut self, top: usize, bottom: usize) {
        self.top = top;
        self.bottom = bottom;
    }

    pub fn advance_cursor(&mut self, wrap_on_end: bool) {
        let cursor_at_end = self.cursor.col == (self.cols);
        if cursor_at_end && wrap_on_end {
            self.cursor.down(1, self.rows);
            self.cursor.col = 0;
        } else if !cursor_at_end {
            self.cursor.right(1, self.cols);
        }
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up(n) => self.cursor.up(n, self.rows - 1),
            Direction::Down(n) => self.cursor.down(n, self.rows - 1),
            Direction::Left(n) => self.cursor.left(n, self.cols - 1),
            Direction::Right(n) => self.cursor.right(n, self.cols - 1),
        }
    }
}

impl Buffer<Cell> {
    pub fn write(&mut self, c: char, cell_style: CellStyle) {
        if let Some(cell) = self.get(self.cursor) {
            cell.content = c;
            cell.style = cell_style;
        } else {
            println!("Warning: tried printing outside grid");
        }
    }
}

impl<T> Index<Cursor> for Buffer<T> {
    type Output = T;

    fn index(&self, index: Cursor) -> &Self::Output {
        &self.data[index.row * self.cols + index.col]
    }
}

impl<T> IndexMut<Cursor> for Buffer<T> {
    fn index_mut(&mut self, index: Cursor) -> &mut Self::Output {
        &mut self.data[index.row * self.cols + index.col]
    }
}

impl<T> IndexMut<std::ops::Range<usize>> for Buffer<T> {
    fn index_mut(&mut self, range: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.data[range]
    }
}

impl<T> Index<std::ops::Range<usize>> for Buffer<T> {
    type Output = [T];

    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.data[range]
    }
}

pub enum Selection {
    Line,
    FromStartOfLine,
    ToEndOfLine,
    ToEndOfDisplay,
    Characters(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_grid_retains_correct_width_height() {
        let grid = Buffer::new(10, 5, vec![1; 10 * 5]);
        assert_eq!(grid.cols, 5);
        assert_eq!(grid.rows, 10);
        assert_eq!(grid.data.len(), 5 * 10);
    }

    #[test]
    #[should_panic]
    fn creating_grid_with_wrong_content_length_throws() {
        Buffer::new(10, 5, vec![1; 10 * 6]);
    }

    #[test]
    fn iterating_rows_should_create_correct_row_length() {
        let grid = Buffer::new(3, 5, vec![1; 3 * 5]);
        let rows: Vec<&[i32]> = grid.iter_rows().collect();
        assert_eq!(rows.len(), 3);
        for row in rows {
            assert_eq!(row.len(), 5);
        }
    }

    #[test]
    fn shifting_row_preserves_size_and_adds_empty_row_last() {
        let mut grid = Buffer::new(3, 5, vec![1; 3 * 5]);
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
        let mut grid = Buffer::new(3, 5, vec![1; 3 * 5]);
        grid.resize(2, 5);

        assert_eq!(grid.rows, 2);
        assert_eq!(grid.cols, 5);
        assert_eq!(grid.data, vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn resizing_to_more_rows_works_correctly() {
        let mut grid = Buffer::new(3, 2, vec![1; 3 * 2]);
        grid.resize(4, 2);

        assert_eq!(grid.rows, 4);
        assert_eq!(grid.cols, 2);
        assert_eq!(grid.data, vec![1, 1, 1, 1, 1, 1, 0, 0]);
    }

    #[test]
    fn resizing_to_more_cols_then_fewer_rows_works_correctly() {
        let mut grid = Buffer::new(1, 2, vec![1; 1 * 2]);
        grid.resize(2, 2);
        assert_eq!(grid.data, vec![1, 1, 0, 0]);
        grid.resize(2, 1);
        assert_eq!(grid.data, vec![1, 1]);
    }

    #[test]
    fn moving_cursor_outside_buffer_should_not_crash() {
        let mut grid = Buffer::new(2, 2, vec![1; 2 * 2]);
        grid.move_cursor(Direction::Left(5));
        assert_eq!(grid.cursor, Cursor::default());
    }

    #[test]
    fn scrolling_one_line_down_should_work() {
        let mut grid = Buffer::new(3, 2, vec![1, 2, 3, 4, 5, 6]);
        grid.shift_row();
        assert_eq!(grid.data, vec![3, 4, 5, 6, 0, 0]);
    }
}
