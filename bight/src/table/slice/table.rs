pub mod iter;

use std::fmt::Debug;

use crate::table::{
    Table, TableRef,
    cell::CellPos,
    slice::{
        col::ColSlice,
        row::RowSlice,
        table::iter::{TableColSliceIter, TableRowSliceIter, TableSliceIter},
    },
};

use super::{CellRange, IdxRange};

/// A slice of a table (a wide pointer to a table that has a starting cell and an ending cell)
///
/// The slice is non-inclusive (the end cell, its row and col are not included)
/// Both end's coordinates are greater or equal to the corresponding start's coordinates (end must
/// be to the down-right of the start)
pub struct TableSlice<'a, T> {
    r: TableRef<'a, T>,
    width: usize,
    height: usize,
}

impl<T> Clone for TableSlice<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for TableSlice<'_, T> {}

impl<'a, T: Table> TableSlice<'a, T> {
    pub fn new(r: TableRef<'a, T>, width: usize, height: usize) -> Self {
        Self { r, width, height }
    }
    pub fn slice_table(pos: impl Into<CellRange>, table: &'a T) -> Self {
        let pos = pos.into();
        let r = table.ref_sh(pos.start);
        Self {
            r,
            width: pos.width,
            height: pos.height,
        }
    }

    pub fn get(&self, pos: impl Into<CellPos>) -> Option<&'a T::Item> {
        let pos: CellPos = pos.into();
        if pos.x >= 0 && pos.y >= 0 && pos.x < self.width as isize && pos.y < self.height as isize {
            self.r.offset_get(pos.x, pos.y)
        } else {
            None
        }
    }
    pub fn sep_top_row(self) -> Option<(RowSlice<'a, T>, Self)> {
        if self.height > 0 {
            Some((
                Self {
                    r: self.r,
                    width: self.width,
                    height: 1,
                }
                .try_into()
                .unwrap(),
                Self {
                    r: self.r.offset(0, 1),
                    width: self.width,
                    height: self.height - 1,
                },
            ))
        } else {
            None
        }
    }
    pub fn sep_left_col(self) -> Option<(ColSlice<'a, T>, Self)> {
        if self.width > 0 {
            Some((
                Self {
                    r: self.r,
                    width: 1,
                    height: self.height,
                }
                .try_into()
                .unwrap(),
                Self {
                    r: self.r.offset(1, 0),
                    width: self.width - 1,
                    height: self.height,
                },
            ))
        } else {
            None
        }
    }
    pub fn start(&self) -> CellPos {
        self.r.pos()
    }
    pub fn end(&self) -> CellPos {
        CellPos {
            x: self.r.pos().x + self.width as isize,
            y: self.r.pos().y + self.height as isize,
        }
    }
    pub fn is_col(&self) -> bool {
        self.width == 1
    }

    pub fn is_row(&self) -> bool {
        self.height == 1
    }

    pub fn row_indexes(&self) -> IdxRange {
        0..self.height as isize
    }

    pub fn col_indexes(&self) -> IdxRange {
        0..self.width as isize
    }

    pub fn rows(self) -> TableRowSliceIter<'a, T> {
        self.into()
    }

    pub fn cols(self) -> TableColSliceIter<'a, T> {
        self.into()
    }

    pub fn cells(self) -> TableSliceIter<'a, T> {
        self.into_iter()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl<'a, T: Table> Debug for TableSlice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let end_pos = CellPos {
            x: self.r.pos().x + self.width as isize,
            y: self.r.pos().y + self.height as isize,
        };
        write!(f, "TableSlice {}..{}", self.r.pos(), end_pos)
    }
}
