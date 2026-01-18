//! The module with funtions and data types for saving and loading files, as well as other means of
//! importing and exporting bight tables. Files may be saved/loaded based on their extension with
//! [`save`] and [`load`], or as a known filetype with the corresponding functions in the
//! submodules. Some filetypes may also be imported and exported as bytes or as a string.

pub mod bight;
pub mod csv;

pub use bight::BightFile;
pub use bight::{load as load_bight, save as save_bight};
pub use csv::load as load_csv;
pub use csv::slice_to_csv_string;

use std::path::Path;

/// The error type for conversion from bytes to a bight file
#[derive(Debug, thiserror::Error)]
pub enum DeserializationError {
    #[error("The length of the data is less that the minimum requirement for the header")]
    InvalidLength,
    #[error(transparent)]
    ArchiveError(#[from] rkyv::rancor::Error),
    #[error("Data contains invalid csv")]
    CsvError,
    #[error("Bight file version {0} is not supported")]
    UnsupportedVersion(u64),
}

/// The error type for loading a file
#[derive(Debug, thiserror::Error)]
pub enum FileLoadError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
    #[error("The filetype {0} is not supported")]
    UnsupportedFiletype(String),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
}

/// The error type for saving a file
#[derive(Debug, thiserror::Error)]
pub enum FileSaveError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
    #[error("The filetype {0} is not supported")]
    UnsupportedFiletype(String),
}

/// Loads a file, guessing the filetype based on the file extension. For specialized functions for
/// loading a file of a known type see `load` functions in submodules, or their reexports
/// [`load_bight`], [`load_csv`], etc. Return an error if the extension is not supported or the
/// file is invalid
pub fn load(path: &Path) -> Result<BightFile, FileLoadError> {
    match path
        .extension()
        .ok_or(FileLoadError::UnsupportedFiletype(String::from("")))?
        .to_str()
        .ok_or(FileLoadError::UnsupportedFiletype(String::from("")))?
    {
        "bight" => bight::load(path),
        "csv" => csv::load(path),
        ext => Err(FileLoadError::UnsupportedFiletype(ext.to_owned())),
    }
}

/// Saves a file, guessing the filetype based on the file extension. For specialized functions for
/// saving a file of a known type see `save` functions in submodules, or their reexports
/// [`save_bight`], etc. Return an error if the extension is not supported or the
/// file couldn't be saved
pub fn save(path: &Path, file: &BightFile) -> Result<(), FileSaveError> {
    match path
        .extension()
        .ok_or(FileSaveError::UnsupportedFiletype(String::from("")))?
        .to_str()
        .ok_or(FileSaveError::UnsupportedFiletype(String::from("")))?
    {
        "bight" => Ok(bight::save(path, file)?),
        ext => Err(FileSaveError::UnsupportedFiletype(ext.to_owned())),
    }
}
