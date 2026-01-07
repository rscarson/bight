use crate::table::{CellPos, Table, TableMut, TableSlice};

#[derive(Debug)]
struct Cell<I> {
    content: Option<I>,
}

impl<I> Default for Cell<I> {
    fn default() -> Self {
        Self { content: None }
    }
}

#[derive(Debug)]
pub struct DataTable<I> {
    data: Vec<Vec<Cell<I>>>,
}

impl<I> DataTable<I> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes a table slice that is guaranteed to contain every set element of this table (but
    /// doesn't guarantee that every element of slice is set)
    pub fn full_slice(&self) -> TableSlice<'_, Self> {
        let rows = self.data.len();
        let cols = self.data.iter().map(|v| v.len()).max().unwrap_or(0);
        TableSlice::slice_table(((0, 0), (rows as isize, cols as isize)), self)
    }
}

impl<I> Default for DataTable<I> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<I> Table for DataTable<I> {
    type Item = I;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        if pos.x.is_negative() || pos.y.is_negative() {
            return None;
        }
        self.data
            .get(pos.x as usize)?
            .get(pos.y as usize)?
            .content
            .as_ref()
    }
}

impl<I> TableMut for DataTable<I> {
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item> {
        if pos.x.is_negative() || pos.y.is_negative() {
            return None;
        }
        let x = pos.x as usize;
        let y = pos.y as usize;
        self.data.get_mut(x)?.get_mut(y)?.content.as_mut()
    }
    fn set(&mut self, pos: CellPos, item: Option<Self::Item>) {
        if pos.x.is_negative() || pos.y.is_negative() {
            panic!(
                "Attempted to index DataTable with negative cell coordinates: {}",
                pos
            );
        }
        let x = pos.x as usize;
        let y = pos.y as usize;
        if self.data.len() <= x {
            self.data.resize_with(x + 1, Vec::default);
        }
        if self.data[x].len() <= y {
            self.data[x].resize_with(y + 1, Cell::default);
        }

        self.data[x][y].content = item;
    }
}
