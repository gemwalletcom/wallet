use gem_encoding::borsh::{DecodeError, Reader};

use crate::Pubkey;

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Data {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<Creator>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

impl Data {
    pub(super) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            name: reader.string()?,
            symbol: reader.string()?,
            uri: reader.string()?,
            seller_fee_basis_points: reader.u16()?,
            creators: reader.option(|reader| reader.vec(Creator::read))?,
        })
    }
}

impl Creator {
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            address: Pubkey::read(reader)?,
            verified: reader.bool()?,
            share: reader.u8()?,
        })
    }
}
