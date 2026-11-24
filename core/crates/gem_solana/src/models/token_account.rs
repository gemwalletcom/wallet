use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};

use crate::{SolanaDataSlice, SolanaError, error::Result as SolanaResult};

const AMOUNT_OFFSET: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenAccount {
    pub amount: u64,
}

impl BorshDeserialize for TokenAccount {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self { amount: reader.skip(AMOUNT_OFFSET)?.read()? })
    }
}

impl TokenAccount {
    pub const AMOUNT_SLICE: SolanaDataSlice = SolanaDataSlice {
        offset: AMOUNT_OFFSET,
        length: size_of::<u64>(),
    };

    pub fn decode(data: &[u8]) -> SolanaResult<Self> {
        Self::deserialize(&mut &data[..]).map_err(|_| SolanaError::invalid_input("Invalid token account"))
    }

    pub fn decode_amount(slice: &[u8]) -> SolanaResult<u64> {
        u64::try_from_slice(slice).map_err(|_| SolanaError::invalid_input("Invalid token amount"))
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
