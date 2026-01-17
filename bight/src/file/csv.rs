use std::{fmt::Display, io::Write, path::Path, sync::Arc};

use crate::{
    file::{BightFile, DeserializationError, FileLoadError},
    table::{CellPos, Table, slice::table::TableSlice},
};

/// Converst the given table slice into csv using the given writer
pub fn write_slice_to_csv(
    slice: TableSlice<'_, impl Table<Item: Display>>,
    writer: &mut csv::Writer<impl Write>,
) -> Result<(), csv::Error> {
    for x in slice.cols() {
        writer.write_record(
            x.into_iter()
                .map(|v| v.map(|v| v.to_string()).unwrap_or_default()),
        )?;
    }
    Ok(())
}

/// Converst the given table slice into a String of comma-separated values
pub fn slice_to_csv_string(slice: TableSlice<'_, impl Table<Item: Display>>) -> String {
    let mut s = Vec::<u8>::new();
    let mut v = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(&mut s);

    write_slice_to_csv(slice, &mut v).expect("The writer configuration was valid");
    v.flush().expect("The writer configuration is valid");
    drop(v);

    String::from_utf8(s).expect("No non-utf8 data was written")
}

/// Converts the given bytes into a BightFile, interpreting the bytes as a csv file
pub fn from_bytes(bytes: &[u8]) -> Result<BightFile, DeserializationError> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes);

    let source = reader
        .into_records()
        .enumerate()
        .map(|(posy, record)| {
            Ok(record?
                .into_iter()
                .enumerate()
                .map(move |(posx, value)| {
                    let pos = CellPos::from((posx as isize, posy as isize));
                    (pos, Arc::from(value))
                })
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, csv::Error>>()
        .map_err(|_| DeserializationError::CsvError)?
        .into_iter()
        .flatten()
        .collect();
    Ok(BightFile { source })
}

// pub fn save(path: &Path, file: &BightFile) -> Result<(), FileSaveError> {
//     let end = CellPos::from((0, 0));
//
// }

/// Loads a csv file
pub fn load(path: &Path) -> Result<BightFile, FileLoadError> {
    let bytes = std::fs::read(path)?;
    Ok(from_bytes(&bytes)?)
}

#[cfg(test)]
mod test {
    use crate::table::{DataTable, TableMut};

    use super::*;

    #[test]
    fn csv() {
        let mut table = DataTable::new();
        table.set((0, 0).into(), Some("Hello, "));
        table.set((1, 1).into(), Some("World!"));

        let csv = slice_to_csv_string(table.full_slice());

        assert_eq!(csv, "\"Hello, \",\n,World!\n");
    }

    #[test]
    fn cyrillic() {
        let mut table = DataTable::new();
        table.set((0, 0).into(), Some("Привет, "));
        table.set((1, 1).into(), Some("мир!"));

        let csv = slice_to_csv_string(table.full_slice());

        assert_eq!(csv, "\"Привет, \",\n,мир!\n");
    }
}
