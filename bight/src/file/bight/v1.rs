use rkyv::{Archive, Deserialize, Serialize, access, deserialize, to_bytes, util::AlignedVec};

use crate::{evaluator::SourceTable, file::bight::DeserializationError};

#[derive(Archive, Serialize, Deserialize)]
pub struct BightFile {
    pub source: SourceTable,
}

impl BightFile {
    pub const VERSION: u64 = 1;
    pub fn empty() -> Self {
        Self {
            source: SourceTable::new(),
        }
    }
    pub fn new(source: SourceTable) -> Self {
        Self { source }
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializationError> {
        let archived = access::<ArchivedBightFile, rkyv::rancor::Error>(bytes)?;
        Ok(deserialize::<BightFile, rkyv::rancor::Error>(archived)?)
    }
    pub fn to_bytes(&self) -> AlignedVec {
        to_bytes::<rkyv::rancor::Error>(self).unwrap()
    }
}
