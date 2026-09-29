use crate::{AddressLookupTableAccount, Pubkey};

impl AddressLookupTableAccount {
    pub(crate) fn mock(table_key: &str, entries: &[(u8, &str)]) -> Self {
        let max_index = entries.iter().map(|(index, _)| *index).max().unwrap_or(0) as usize;
        let mut addresses: Vec<Pubkey> = (0..=max_index)
            .map(|entry_index| {
                let mut bytes = [0; 32];
                bytes[0] = 0xFE;
                bytes[1..3].copy_from_slice(&(entry_index as u16).to_le_bytes());
                Pubkey::new(bytes)
            })
            .collect();

        for (index, value) in entries {
            addresses[*index as usize] = Pubkey::from_base58(value).unwrap();
        }

        Self::new(Pubkey::from_base58(table_key).unwrap(), addresses)
    }
}
