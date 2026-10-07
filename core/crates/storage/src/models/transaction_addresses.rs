use std::collections::HashMap;

use diesel::prelude::*;
use primitives::Transaction;

use crate::sql_types::{AssetId, ChainRow};

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::transactions_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewTransactionAddressesRow {
    pub address_id: i32,
    pub transaction_id: i64,
    pub asset_id: AssetId,
}

impl NewTransactionAddressesRow {
    pub fn from_transaction(transaction_id: i64, transaction: &Transaction, address_ids: &HashMap<String, i32>) -> Vec<NewTransactionAddressesRow> {
        transaction
            .assets_addresses()
            .into_iter()
            .filter_map(|asset_address| {
                address_ids.get(&asset_address.address).map(|&address_id| Self {
                    address_id,
                    transaction_id,
                    asset_id: asset_address.asset_id.into(),
                })
            })
            .collect()
    }
}

#[derive(Queryable, Debug, Clone)]
pub(crate) struct AddressChainIdResultRow {
    pub address: String,
    pub chain_id: ChainRow,
}
