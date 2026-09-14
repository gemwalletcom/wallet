use crate::{Result, SolanaError, types::Pubkey};

const LOOKUP_TABLE_META_SIZE: usize = 56;
const LOOKUP_TABLE_DISCRIMINANT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageAddressTableLookup {
    pub account_key: Pubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

impl MessageAddressTableLookup {
    pub fn new(account_key: Pubkey, writable_indexes: Vec<u8>, readonly_indexes: Vec<u8>) -> Self {
        Self {
            account_key,
            writable_indexes,
            readonly_indexes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressLookupTableAccount {
    pub key: Pubkey,
    pub addresses: Vec<Pubkey>,
}

impl AddressLookupTableAccount {
    pub fn new(key: Pubkey, addresses: Vec<Pubkey>) -> Self {
        Self { key, addresses }
    }

    pub fn from_account_data(key: Pubkey, data: &[u8]) -> Result<Self> {
        if data.len() < LOOKUP_TABLE_META_SIZE {
            return Err(SolanaError::InvalidMessage);
        }

        let discriminant = u32::from_le_bytes(data[0..4].try_into().map_err(|_| SolanaError::InvalidMessage)?);
        if discriminant != LOOKUP_TABLE_DISCRIMINANT {
            return Err(SolanaError::InvalidMessage);
        }

        let address_data = &data[LOOKUP_TABLE_META_SIZE..];
        if !address_data.len().is_multiple_of(32) {
            return Err(SolanaError::InvalidMessage);
        }

        let addresses = address_data
            .chunks_exact(32)
            .map(|chunk| chunk.try_into().map(Pubkey::new).map_err(|_| SolanaError::InvalidMessage))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { key, addresses })
    }
}

#[cfg(test)]
mod tests {
    use hex_lit::hex;

    use super::*;
    use crate::types::Pubkey;

    #[test]
    fn test_address_lookup_table_from_account_data() {
        let key = Pubkey::new([9; 32]);
        let mut data = vec![0u8; LOOKUP_TABLE_META_SIZE];
        data[0..4].copy_from_slice(&LOOKUP_TABLE_DISCRIMINANT.to_le_bytes());
        data.extend_from_slice(&[2u8; 32]);
        data.extend_from_slice(&[3u8; 32]);

        let parsed = AddressLookupTableAccount::from_account_data(key, &data).unwrap();

        assert_eq!(parsed, AddressLookupTableAccount::new(key, vec![Pubkey::new([2u8; 32]), Pubkey::new([3u8; 32])]));
    }

    #[test]
    fn test_address_lookup_table_from_account_data_rejects_invalid_length() {
        let key = Pubkey::new([9; 32]);
        let mut invalid_data = vec![0u8; LOOKUP_TABLE_META_SIZE + 1];
        invalid_data[..4].copy_from_slice(&LOOKUP_TABLE_DISCRIMINANT.to_le_bytes());

        let result = AddressLookupTableAccount::from_account_data(key, &invalid_data);

        assert_eq!(result.unwrap_err(), SolanaError::InvalidMessage);
    }

    #[test]
    fn test_address_lookup_table_from_account_data_rejects_invalid_discriminant() {
        let key = Pubkey::new([9; 32]);
        let mut data = vec![0u8; LOOKUP_TABLE_META_SIZE];
        data[0..4].copy_from_slice(&hex!("deadbeef"));
        data.extend_from_slice(&[2u8; 32]);

        let result = AddressLookupTableAccount::from_account_data(key, &data);

        assert_eq!(result.unwrap_err(), SolanaError::InvalidMessage);
    }
}
