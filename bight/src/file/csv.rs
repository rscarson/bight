use std::{fmt::Display, io::Write, path::Path, sync::Arc};

use crate::{
    evaluator::SourceTable,
    file::{BightFile, DeserializationError, FileLoadError},
    table::{CellPos, Table, TableMut, TableRefMut, slice::table::TableSlice},
};

/// Converst the given table slice into csv using the given writer
pub fn write_slice_to_csv(
    slice: TableSlice<'_, impl Table<Item: Display>>,
    writer: &mut csv::Writer<impl Write>,
) -> Result<(), csv::Error> {
    for x in slice.rows() {
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
    Ok(BightFile {
        source: SourceTable::from_source(source),
    })
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

pub fn load_into<T: TableMut<Item: From<Arc<str>>> + ?Sized>(
    path: &Path,
    mut table: TableRefMut<'_, T>,
) -> Result<(), FileLoadError> {
    let file = load(path)?;

    for (pos, source) in file.source.into_inner_iter() {
        table.set(pos, Some(source.into()));
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn csv() {
        let table = small_latin_str_data_table();

        let csv = slice_to_csv_string(table.full_slice());

        assert_eq!(csv, "Hello ,\n\"again, \",World!\n");
    }

    #[test]
    fn cyrillic() {
        let table = small_cyrillic_str_data_table();

        let csv = slice_to_csv_string(table.full_slice());

        assert_eq!(csv, "Привет ,\n\"снова, \",мир!\n");
    }
}
