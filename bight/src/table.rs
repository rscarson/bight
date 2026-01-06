pub mod cell;
pub mod data_table;
pub mod reference;
pub mod slice;

pub use cell::CellPos;
pub use data_table::DataTable;
pub use reference::{TableRef, TableRefMut};
pub use slice::CellRange;
pub use slice::table::TableSlice;

use hashbrown::HashMap;

pub type HashTable<T> = HashMap<CellPos, T>;

pub trait Table {
    type Item;
    fn get(&self, pos: CellPos) -> Option<&Self::Item>;
    fn ref_sh(&self, pos: CellPos) -> TableRef<'_, Self> {
        TableRef::new(self, pos)
    }
}

pub trait TableMut: Table {
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item>;
    fn set(&mut self, pos: CellPos, item: Option<Self::Item>);
    fn ref_mut(&mut self, pos: CellPos) -> TableRefMut<'_, Self> {
        TableRefMut::new(self, pos)
    }
}
