use std::collections::HashMap;

use crate::{AccountMeta, AddressLookupTableAccount, MessageAddressTableLookup, Pubkey};

use super::accounts::MAX_ACCOUNT_KEYS;

pub(super) struct LoadedAccounts {
    writable: Vec<Vec<(Pubkey, u8)>>,
    readonly: Vec<Vec<(Pubkey, u8)>>,
}

impl LoadedAccounts {
    pub(super) fn new(table_count: usize) -> Self {
        Self {
            writable: vec![Vec::new(); table_count],
            readonly: vec![Vec::new(); table_count],
        }
    }

    pub(super) fn push(&mut self, account: AccountMeta, (table_index, entry_index): (usize, u8)) {
        let accounts = if account.is_writable { &mut self.writable[table_index] } else { &mut self.readonly[table_index] };
        accounts.push((account.pubkey, entry_index));
    }

    pub(super) fn len(&self) -> usize {
        self.writable.iter().chain(&self.readonly).map(Vec::len).sum()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &(Pubkey, u8)> {
        self.writable.iter().flatten().chain(self.readonly.iter().flatten())
    }

    pub(super) fn message_lookups(&self, tables: &[AddressLookupTableAccount]) -> Vec<MessageAddressTableLookup> {
        tables
            .iter()
            .enumerate()
            .filter_map(|(table_index, table)| {
                let writable_indexes = self.writable[table_index].iter().map(|(_, index)| *index).collect::<Vec<_>>();
                let readonly_indexes = self.readonly[table_index].iter().map(|(_, index)| *index).collect::<Vec<_>>();
                (!writable_indexes.is_empty() || !readonly_indexes.is_empty()).then(|| MessageAddressTableLookup::new(table.key, writable_indexes, readonly_indexes))
            })
            .collect()
    }
}

pub(super) fn lookup_locations(address_lookup_tables: &[AddressLookupTableAccount]) -> HashMap<Pubkey, (usize, u8)> {
    let mut locations = HashMap::new();
    for (table_index, table) in address_lookup_tables.iter().enumerate().rev() {
        for (entry_index, address) in table.addresses.iter().take(MAX_ACCOUNT_KEYS).enumerate() {
            locations.insert(*address, (table_index, entry_index as u8));
        }
    }
    locations
}
