use crate::table::{CellPos, Table, TableMut};

pub struct TableRef<'a, T: ?Sized> {
    table: &'a T,
    pos: CellPos,
}

impl<T: ?Sized> Clone for TableRef<'_, T> {
    fn clone(&self) -> Self {
        *self
        // Self {
        //     table: self.table,
        //     pos: self.pos,
        // }
    }
}

impl<T: ?Sized> Copy for TableRef<'_, T> {}

impl<'a, T: Table + ?Sized> TableRef<'a, T> {
    pub fn new(table: &'a T, pos: CellPos) -> Self {
        Self { table, pos }
    }
    pub fn pos(&self) -> CellPos {
        self.pos
    }
    pub fn get_self(&self) -> Option<&'a T::Item> {
        self.table.get(self.pos)
    }
    pub fn offset(self, offset_x: isize, offset_y: isize) -> Self {
        Self {
            table: self.table,
            pos: CellPos {
                x: self.pos.x + offset_x,
                y: self.pos.y + offset_y,
            },
        }
    }
    pub fn offset_get(&self, offset_x: isize, offset_y: isize) -> Option<&'a T::Item> {
        let mut pos = self.pos;
        pos.x += offset_x;
        pos.y += offset_y;
        self.table.get(pos)
    }
}

impl<'a, T: Table + ?Sized> Table for TableRef<'a, T> {
    type Item = T::Item;
    fn get(&self, pos: CellPos) -> Option<&'a Self::Item> {
        self.table.get(self.pos + pos)
    }
}

pub struct TableRefMut<'a, T: ?Sized> {
    table: &'a mut T,
    pos: CellPos,
}
impl<'a, T: TableMut + ?Sized> TableRefMut<'a, T> {
    pub fn new(table: &'a mut T, pos: CellPos) -> Self {
        Self { table, pos }
    }
    pub fn pos(&self) -> CellPos {
        self.pos
    }
    pub fn get_self(&'a self) -> Option<&'a T::Item> {
        self.table.get(self.pos)
    }
    pub fn offset_get(&'a self, offset_x: isize, offset_y: isize) -> Option<&'a T::Item> {
        let mut pos = self.pos;
        pos.x += offset_x;
        pos.y += offset_y;
        self.table.get(pos)
    }
    pub fn get_mut_self(&'a mut self) -> Option<&'a mut T::Item> {
        self.table.get_mut(self.pos)
    }
    pub fn offset(self, offset_x: isize, offset_y: isize) -> Self {
        Self {
            table: self.table,
            pos: CellPos {
                x: self.pos.x + offset_x,
                y: self.pos.y + offset_y,
            },
        }
    }
    pub fn offset_get_mut(
        &'a mut self,
        offset_x: isize,
        offset_y: isize,
    ) -> Option<&'a mut T::Item> {
        let mut pos = self.pos;
        pos.x += offset_x;
        pos.y += offset_y;
        self.table.get_mut(pos)
    }
}
impl<'a, T: Table + ?Sized> Table for TableRefMut<'a, T> {
    type Item = T::Item;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        self.table.get(self.pos + pos)
    }
}

impl<'a, T: TableMut + ?Sized> TableMut for TableRefMut<'a, T> {
    fn get_mut(&mut self, pos: CellPos) -> Option<&mut Self::Item> {
        self.table.get_mut(self.pos + pos)
    }
    fn set(&mut self, mut pos: CellPos, item: Option<Self::Item>) {
        pos += self.pos;
        self.table.set(pos, item);
    }
}

impl<'a, T> From<TableRefMut<'a, T>> for TableRef<'a, T> {
    fn from(value: TableRefMut<'a, T>) -> Self {
        Self {
            table: value.table,
            pos: value.pos,
        }
    }
}
