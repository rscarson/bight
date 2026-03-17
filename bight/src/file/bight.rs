mod v1;
use std::path::Path;

use rkyv::{Archive, Deserialize, Serialize, access};
pub use v1::BightFile;
use v1::BightFile as BightFileV1;

use crate::{
    file::{DeserializationError, FileLoadError},
    sync::RcStr,
    table::{TableMut, TableRefMut},
};

/// Contents of a header of a .bight file.  
///
/// Backwards compatability: new fields may only be added at the end. Any fields must be valid if initialized with 0. If a header from an earlier version is being read by a newer version of bight, any missing fields are initialized to be 0. BightHeaderPadded should be used to store to ensure that the header's size doesn't change between versions and can always be read with defined contents.
#[derive(Archive, Serialize, Deserialize)]
#[repr(C)]
pub struct BightHeader {
    version: u64,
}

const PADDED_HEADER_SIZE: usize = 1024;
const RESERVED_SIZE: usize = PADDED_HEADER_SIZE - std::mem::size_of::<BightHeader>();

/// Header of a .bight file. Padding is required for backwards compatability. Any addidional data
/// that does not fit in the header should be in the contents, which may have variable size.
#[derive(Archive, Serialize, Deserialize)]
#[repr(C)]
struct BightHeaderPadded {
    header: BightHeader,
    _reserved: [u8; RESERVED_SIZE],
}

#[bon::bon]
impl BightHeaderPadded {
    #[builder]
    fn new(version: u64) -> Self {
        Self {
            header: BightHeader { version },
            _reserved: [0; RESERVED_SIZE],
        }
    }
}

/// Converts the bytes into the latest version of bight file. Returns new empty file if `bytes` is empty
pub fn from_bytes(bytes: &[u8]) -> Result<BightFile, DeserializationError> {
    if bytes.is_empty() {
        return Ok(BightFile::empty());
    }

    let (header_bytes, data_bytes) = bytes
        .split_at_checked(PADDED_HEADER_SIZE)
        .ok_or(DeserializationError::InvalidLength)?;

    let archived_header = access::<ArchivedBightHeaderPadded, rkyv::rancor::Error>(header_bytes)?;
    let version = archived_header.header.version.to_native();

    match version {
        BightFileV1::VERSION => BightFileV1::from_bytes(data_bytes),
        _ => Err(DeserializationError::UnsupportedVersion(version)),
    }
}

/// Load a bight file
pub fn load(path: &Path) -> Result<BightFile, FileLoadError> {
    let bytes = std::fs::read(path)?;
    Ok(from_bytes(&bytes)?)
}

pub fn load_into<T: TableMut<Item: From<RcStr>> + ?Sized>(
    path: &Path,
    mut table: TableRefMut<'_, T>,
) -> Result<(), FileLoadError> {
    let file = load(path)?;

    for (pos, source) in file.source.into_inner_iter() {
        table.set(pos, Some(source.into()));
    }

    Ok(())
}

/// Converst a bight file into bytes for storage
pub fn to_bytes(file: &BightFile) -> rkyv::util::AlignedVec {
    let header = BightHeaderPadded::builder()
        .version(BightFile::VERSION)
        .build();
    let mut bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&header).unwrap();

    bytes.extend_from_slice(&file.to_bytes());
    bytes
}

/// Saves a bight file
pub fn save(path: &Path, file: &BightFile) -> Result<(), std::io::Error> {
    let bytes = to_bytes(file);
    std::fs::write(path, bytes)?;
    Ok(())
}
