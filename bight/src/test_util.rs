use crate::table::{DataTable, TableMut as _};

// const Path::new() is unstable as of 1.92
pub static TEST_OUTPUT_PATH: &str = "../test_output/";

pub fn small_latin_str_data_table() -> DataTable<&'static str> {
    let mut table = DataTable::new();
    table.set((0, 0).into(), Some("Hello "));
    table.set((0, 1).into(), Some("again, "));
    table.set((1, 1).into(), Some("World!"));
    table
}

pub fn small_cyrillic_str_data_table() -> DataTable<&'static str> {
    let mut table = DataTable::new();
    table.set((0, 0).into(), Some("Привет "));
    table.set((0, 1).into(), Some("снова, "));
    table.set((1, 1).into(), Some("мир!"));
    table
}

pub fn normal_float_data_table() -> DataTable<f64> {
    let mut table = DataTable::new();
    table.set((0, 0).into(), Some(-1.0));
    table.set((1, 0).into(), Some(0.0));
    table.set((2, 0).into(), Some(1.0));

    table.set((0, 1).into(), Some(0.0));
    table.set((1, 1).into(), Some(0.0));
    table.set((2, 1).into(), Some(1.5));

    table.set((0, 2).into(), Some(1.0));
    table.set((1, 2).into(), Some(1.0));
    table.set((2, 2).into(), Some(1.0));

    table.set((0, 3).into(), Some(2.0));
    table.set((1, 3).into(), Some(-1.0));
    table.set((2, 3).into(), Some(0.5));

    table.set((0, 4).into(), Some(3.0));
    table.set((1, 4).into(), Some(-1.0));
    table.set((1, 4).into(), Some(2.0));

    table
}
