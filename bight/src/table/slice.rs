pub mod col;
pub mod row;
pub mod table;

use std::{ops::Range, str::FromStr};

use crate::table::cell::CellPosParseError;

use super::cell::CellPos;

pub type IdxRange = Range<isize>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellPos,
    pub width: usize,
    pub height: usize,
}

impl CellRange {
    pub fn new_limits<A: Into<CellPos>, B: Into<CellPos>>(start: A, end: B) -> Self {
        let mut start: CellPos = start.into();
        let mut end: CellPos = end.into();

        if start.x > end.x {
            std::mem::swap(&mut start.x, &mut end.x);
        }
        if start.y > end.y {
            std::mem::swap(&mut start.y, &mut end.y);
        }

        Self {
            start,
            width: (end.x - start.x) as usize,
            height: (end.y - start.y) as usize,
        }
    }
    pub fn is_inside(&self, pos: impl Into<CellPos>) -> bool {
        let p: CellPos = pos.into();
        (p.x >= self.start.x)
            && (p.y >= self.start.y)
            && (p.x < self.start.x + self.width as isize)
            && (p.y < self.start.y + self.height as isize)
    }

    pub fn is_valid_shift(&self, shift: CellPos) -> bool {
        let pos: CellPos = (self.start.x + shift.x, self.start.y + shift.y).into();
        self.is_inside(pos)
    }

    pub fn shift_to_pos(&self, shift: CellPos) -> Option<CellPos> {
        let pos: CellPos = (self.start.x + shift.x, self.start.y + shift.y).into();
        self.is_inside(pos).then_some(pos)
    }

    pub fn columns(&self) -> IdxRange {
        0..(self.width as isize)
    }

    pub fn rows(&self) -> IdxRange {
        0..(self.height as isize)
    }
}

impl<A: Into<CellPos>, B: Into<CellPos>> From<(A, B)> for CellRange {
    fn from(value: (A, B)) -> Self {
        Self::new_limits(value.0, value.1)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("given str was not a valid SlicePos")]
pub struct SlicePosParseError;
impl From<CellPosParseError> for SlicePosParseError {
    fn from(_value: CellPosParseError) -> Self {
        SlicePosParseError
    }
}

impl FromStr for CellRange {
    type Err = SlicePosParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cells = s.split('_');
        let mut pos1 = CellPos::from_str(cells.next().ok_or(SlicePosParseError)?)?;
        let mut pos2 = CellPos::from_str(cells.next().ok_or(SlicePosParseError)?)?;
        if pos1.x > pos2.x {
            std::mem::swap(&mut pos1.x, &mut pos2.x);
        }
        if pos1.y > pos2.y {
            std::mem::swap(&mut pos1.y, &mut pos2.y);
        }

        pos2.x += 1;
        pos2.y += 1;

        if cells.next().is_some() {
            return Err(SlicePosParseError);
        }
        Ok((pos1, pos2).into())
    }
}
