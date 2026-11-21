use gem_encoding::borsh::Reader;

use crate::{SolanaDataSlice, SolanaError, error::Result};

const AMOUNT_OFFSET: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenAccount {
    _mint_and_owner: [u8; AMOUNT_OFFSET],
    pub amount: u64,
}

impl TokenAccount {
    pub const AMOUNT_SLICE: SolanaDataSlice = SolanaDataSlice {
        offset: AMOUNT_OFFSET,
        length: size_of::<u64>(),
    };

    pub fn decode(data: &[u8]) -> Result<Self> {
        Reader::decode_prefix(data, |reader| {
            Ok(Self {
                _mint_and_owner: reader.bytes()?,
                amount: reader.u64()?,
            })
        })
        .map_err(|_| SolanaError::invalid_input("Invalid token account"))
    }

    pub fn decode_amount(slice: &[u8]) -> Result<u64> {
        Reader::decode(slice, Reader::u64).map_err(|_| SolanaError::invalid_input("Invalid token amount"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::decode_base64;

    #[test]
    fn test_decode() {
        let data = decode_base64(include_str!("../../testdata/token_account_wsol.base64").trim()).unwrap();
        let SolanaDataSlice { offset, length } = TokenAccount::AMOUNT_SLICE;

        assert_eq!(TokenAccount::decode(&data).unwrap().amount, 931_082_454_136);
        assert_eq!(TokenAccount::decode(&data[..71]), Err(SolanaError::invalid_input("Invalid token account")));
        assert_eq!(TokenAccount::decode_amount(&data[offset..offset + length]), Ok(931_082_454_136));
        assert_eq!(TokenAccount::decode_amount(&data[offset..offset + length - 1]), Err(SolanaError::invalid_input("Invalid token amount")));
    }
}
