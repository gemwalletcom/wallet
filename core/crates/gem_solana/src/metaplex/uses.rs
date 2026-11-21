use gem_encoding::borsh::{DecodeError, Reader};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

impl UseMethod {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match reader.u8()? {
            0 => Ok(Self::Burn),
            1 => Ok(Self::Multiple),
            2 => Ok(Self::Single),
            tag => Err(DecodeError::UnexpectedTag(tag)),
        }
    }
}

impl Uses {
    pub(super) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            use_method: UseMethod::read(reader)?,
            remaining: reader.u64()?,
            total: reader.u64()?,
        })
    }
}
