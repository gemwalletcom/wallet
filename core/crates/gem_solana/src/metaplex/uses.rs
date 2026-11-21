use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

impl BorshDeserialize for UseMethod {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.read::<u8>()? {
            0 => Ok(Self::Burn),
            1 => Ok(Self::Multiple),
            2 => Ok(Self::Single),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

impl BorshDeserialize for Uses {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            use_method: reader.read()?,
            remaining: reader.read()?,
            total: reader.read()?,
        })
    }
}
