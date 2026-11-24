use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};

use crate::Pubkey;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

impl BorshDeserialize for Collection {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            verified: reader.read()?,
            key: reader.read()?,
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollectionDetails {
    V1 { size: u64 },
    V2 { padding: [u8; 8] },
}

impl BorshDeserialize for CollectionDetails {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(Self::V1 { size: reader.read()? }),
            1 => Ok(Self::V2 { padding: reader.read()? }),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}
