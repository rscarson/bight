pub mod bight;
pub mod csv;
pub use bight::load as load_bight;
pub use csv::slice_to_csv_string;
use std::path::Path;

pub use bight::BightFile;

#[derive(Debug, thiserror::Error)]
pub enum DeserializationError {
    #[error("The length of the data is less that the minimum requirement for the header")]
    InvalidLength,
    #[error(transparent)]
    ArchiveError(#[from] rkyv::rancor::Error),
    #[error("Data contains invalid UTF-8")]
    StringError,
    #[error("Bight file version {0} is not supported")]
    UnsupportedVersion(u64),
}
#[derive(Debug, thiserror::Error)]
pub enum FileLoadError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
    #[error("The filetype {0} is not supported")]
    UnsupportedFiletype(String),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
}

#[derive(Debug, thiserror::Error)]
pub enum FileSaveError {
    #[error(transparent)]
    IoErrror(#[from] std::io::Error),
    #[error("The filetype {0} is not supported")]
    UnsupportedFiletype(String),
}

pub fn load(path: &Path) -> Result<BightFile, FileLoadError> {
    match path
        .extension()
        .ok_or(FileLoadError::UnsupportedFiletype(String::from("")))?
        .to_str()
        .ok_or(FileLoadError::UnsupportedFiletype(String::from("")))?
    {
        "bight" => bight::load(path),
        ext => Err(FileLoadError::UnsupportedFiletype(ext.to_owned())),
    }
}
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
