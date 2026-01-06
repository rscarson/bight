use crate::table::{
    Table,
    slice::{col::ColSlice, row::RowSlice},
};

use super::TableSlice;

pub struct TableSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
    current_x_offset: usize,
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        if value.width() > 0 {
            Self {
                slice: value,
                current_x_offset: 0,
            }
        } else {
            Self {
                slice: TableSlice {
                    r: value.r,
                    width: 0,
                    height: 0,
                },
                current_x_offset: 0,
            }
        }
    }
}

impl<'a, T: Table> Iterator for TableSliceIter<'a, T> {
    type Item = Option<&'a T::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.height == 0 {
            return None;
        }
        if self.current_x_offset >= self.slice.width {
            self.current_x_offset = 0;
            self.slice = self.slice.sep_top_row().unwrap().1;
            if self.slice.height == 0 {
                return None;
            }
        }

        let value = self.slice.get((self.current_x_offset as isize, 0));
        self.current_x_offset += 1;
        Some(value)
    }
}
impl<'a, T: Table> IntoIterator for TableSlice<'a, T> {
    type IntoIter = TableSliceIter<'a, T>;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}
pub struct TableRowSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
}

impl<'a, T: Table> Iterator for TableRowSliceIter<'a, T> {
    type Item = RowSlice<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let (row, rest) = self.slice.sep_top_row()?;
        self.slice = rest;
        Some(row)
    }
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableRowSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        Self { slice: value }
    }
}

pub struct TableColSliceIter<'a, T: Table> {
    slice: TableSlice<'a, T>,
}

impl<'a, T: Table> Iterator for TableColSliceIter<'a, T> {
    type Item = ColSlice<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let (col, rest) = self.slice.sep_left_col()?;
        self.slice = rest;
        Some(col)
    }
}

impl<'a, T: Table> From<TableSlice<'a, T>> for TableColSliceIter<'a, T> {
    fn from(value: TableSlice<'a, T>) -> Self {
        Self { slice: value }
    }
}
