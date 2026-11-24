use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};

use crate::Pubkey;

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Data {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
}

impl BorshDeserialize for Data {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            name: reader.read()?,
            symbol: reader.read()?,
            uri: reader.read()?,
            seller_fee_basis_points: reader.read()?,
            creators: reader.read()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

impl BorshDeserialize for Creator {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            address: reader.read()?,
            verified: reader.read()?,
            share: reader.read()?,
        })
    }
}
