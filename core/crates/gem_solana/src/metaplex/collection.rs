use gem_encoding::borsh::{DecodeError, Reader};

use crate::Pubkey;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollectionDetails {
    V1 { size: u64 },
    V2 { padding: [u8; 8] },
}

impl Collection {
    pub(super) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            verified: reader.bool()?,
            key: Pubkey::read(reader)?,
        })
    }
}

impl CollectionDetails {
    pub(super) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.u8()? {
            0 => Ok(Self::V1 { size: reader.u64()? }),
            1 => Ok(Self::V2 { padding: reader.bytes()? }),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}
