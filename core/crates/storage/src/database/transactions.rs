use std::collections::HashSet;

use crate::{
    DatabaseClient,
    models::*,
    schema::{transactions::dsl as transactions_dsl, transactions_addresses},
    sql_types::{AssetId, TransactionState, TransactionType},
};
use chrono::NaiveDateTime;
use diesel::dsl::{count, exists};
use diesel::prelude::*;
use primitives::{Transaction, TransactionId};

pub enum TransactionFilter {
    States(Vec<TransactionState>),
    Kinds(Vec<TransactionType>),
}

#[derive(Debug, Clone)]
pub enum TransactionUpdate {
    State(TransactionState),
    Kind(TransactionType),
    Metadata(serde_json::Value),
}

pub(crate) trait TransactionsStore {
    fn get_transaction_by_id(&mut self, chain: &str, hash: &str) -> Result<TransactionRow, diesel::result::Error>;
    fn get_transactions_by_hash(&mut self, hash: &str) -> Result<Vec<TransactionRow>, diesel::result::Error>;
    fn get_transaction_exists(&mut self, chain: &str, hash: &str) -> Result<bool, diesel::result::Error>;
    fn upsert_transactions(&mut self, transactions: Vec<Transaction>) -> Result<HashSet<TransactionId>, diesel::result::Error>;
    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        asset_id: Option<String>,
        from_datetime: Option<NaiveDateTime>,
        limit: usize,
    ) -> Result<Vec<TransactionRow>, diesel::result::Error>;
    fn count_transactions_by_addresses(&mut self, addresses: Vec<String>, chains: Vec<String>) -> Result<i64, diesel::result::Error>;
    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64, since: NaiveDateTime) -> Result<Vec<AddressChainIdResultRow>, diesel::result::Error>;
    fn delete_transactions_addresses(&mut self, chain_addresses: Vec<AddressChainIdResultRow>) -> Result<Vec<i64>, diesel::result::Error>;
    fn delete_orphaned_transactions(&mut self, candidate_ids: Vec<i64>) -> Result<usize, diesel::result::Error>;
    fn get_asset_usage_counts(&mut self, since: NaiveDateTime) -> Result<Vec<(AssetId, i64)>, diesel::result::Error>;
    fn get_transactions_by_wallet_since(&mut self, wallet_id: i32, since: NaiveDateTime, filters: Vec<TransactionFilter>) -> Result<Vec<TransactionRow>, diesel::result::Error>;
    fn get_transactions_by_filter(&mut self, filters: Vec<TransactionFilter>, limit: i64) -> Result<Vec<TransactionRow>, diesel::result::Error>;
    fn update_transaction(&mut self, chain: &str, hash: &str, updates: Vec<TransactionUpdate>) -> Result<usize, diesel::result::Error>;
    fn get_addresses_by_chain_and_kind(&mut self, chain: &str, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error>;
}

impl DatabaseClient {
    fn upsert_transaction(connection: &mut PgConnection, transaction: &Transaction) -> Result<(TransactionRow, bool), diesel::result::Error> {
        let new_transaction = NewTransactionRow::from_primitive(transaction.clone());
        let inserted = diesel::insert_into(transactions_dsl::transactions)
            .values(&new_transaction)
            .on_conflict((transactions_dsl::chain, transactions_dsl::hash))
            .do_nothing()
            .returning(TransactionRow::as_returning())
            .get_result(connection)
            .optional()?;

        match inserted {
            Some(transaction) => Ok((transaction, true)),
            None => diesel::update(
                transactions_dsl::transactions
                    .filter(transactions_dsl::chain.eq(&new_transaction.chain))
                    .filter(transactions_dsl::hash.eq(&new_transaction.hash)),
            )
            .set((
                transactions_dsl::from_address.eq(&new_transaction.from_address),
                transactions_dsl::to_address.eq(&new_transaction.to_address),
                transactions_dsl::value.eq(&new_transaction.value),
                transactions_dsl::kind.eq(&new_transaction.kind),
                transactions_dsl::state.eq(&new_transaction.state),
                transactions_dsl::fee.eq(&new_transaction.fee),
                transactions_dsl::fee_asset_id.eq(&new_transaction.fee_asset_id),
                transactions_dsl::memo.eq(&new_transaction.memo),
                new_transaction.metadata.as_ref().map(|metadata| transactions_dsl::metadata.eq(metadata)),
                transactions_dsl::utxo_inputs.eq(&new_transaction.utxo_inputs),
                transactions_dsl::utxo_outputs.eq(&new_transaction.utxo_outputs),
            ))
            .returning(TransactionRow::as_returning())
            .get_result(connection)
            .map(|transaction| (transaction, false)),
        }
    }
}

impl TransactionsStore for DatabaseClient {
    fn get_transaction_by_id(&mut self, chain: &str, hash: &str) -> Result<TransactionRow, diesel::result::Error> {
        use crate::schema::transactions::dsl;
        dsl::transactions
            .filter(dsl::chain.eq(chain))
            .filter(dsl::hash.eq(hash))
            .select(TransactionRow::as_select())
            .first(&mut self.connection)
    }

    fn get_transactions_by_hash(&mut self, transaction_hash: &str) -> Result<Vec<TransactionRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl;

        dsl::transactions
            .filter(dsl::hash.eq(transaction_hash))
            .order(dsl::created_at.desc())
            .select(TransactionRow::as_select())
            .load(&mut self.connection)
    }

    fn get_transaction_exists(&mut self, chain: &str, hash: &str) -> Result<bool, diesel::result::Error> {
        use crate::schema::transactions::dsl;

        diesel::select(diesel::dsl::exists(dsl::transactions.filter(dsl::chain.eq(chain)).filter(dsl::hash.eq(hash)))).get_result(&mut self.connection)
    }

    fn upsert_transactions(&mut self, transactions: Vec<Transaction>) -> Result<HashSet<TransactionId>, diesel::result::Error> {
        self.connection
            .build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|conn: &mut diesel::pg::PgConnection| {
                transactions
                    .into_iter()
                    .map(|transaction| {
                        let (stored, is_inserted) = Self::upsert_transaction(conn, &transaction)?;

                        let addresses = NewTransactionAddressesRow::from_transaction(stored.id, &transaction);
                        if !addresses.is_empty() {
                            use crate::schema::transactions_addresses::dsl as addr_dsl;
                            diesel::insert_into(addr_dsl::transactions_addresses)
                                .values(&addresses)
                                .on_conflict((addr_dsl::transaction_id, addr_dsl::address, addr_dsl::asset_id))
                                .do_nothing()
                                .execute(conn)?;
                        }

                        Ok(is_inserted.then_some(transaction.id))
                    })
                    .collect::<Result<Vec<_>, diesel::result::Error>>()
                    .map(|transaction_ids| transaction_ids.into_iter().flatten().collect())
            })
    }

    fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        filter_asset_id: Option<String>,
        from_datetime: Option<NaiveDateTime>,
        limit: usize,
    ) -> Result<Vec<TransactionRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;

        let mut query = transactions
            .into_boxed()
            .inner_join(transactions_addresses::table)
            .filter(chain.eq_any(chains.clone()))
            .filter(transactions_addresses::address.eq_any(addresses))
            .filter(state.ne(TransactionState::InTransit));

        if let Some(filter_asset) = filter_asset_id {
            query = query.filter(transactions_addresses::asset_id.eq(filter_asset));
        }

        if let Some(datetime) = from_datetime {
            query = query.filter(created_at.gt(datetime).or(updated_at.gt(datetime)));
        }

        query
            .order(created_at.desc())
            .limit(limit as i64)
            .select(TransactionRow::as_select())
            .distinct()
            .load(&mut self.connection)
    }

    fn count_transactions_by_addresses(&mut self, addresses: Vec<String>, chains: Vec<String>) -> Result<i64, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;

        if addresses.is_empty() || chains.is_empty() {
            return Ok(0);
        }

        transactions
            .inner_join(transactions_addresses::table)
            .filter(chain.eq_any(chains))
            .filter(transactions_addresses::address.eq_any(addresses))
            .filter(state.ne(TransactionState::InTransit))
            .select(count(id).aggregate_distinct())
            .first(&mut self.connection)
    }

    fn get_transactions_addresses(&mut self, min_count: i64, limit: i64, since: NaiveDateTime) -> Result<Vec<AddressChainIdResultRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl as tx_dsl;
        use crate::schema::transactions_addresses::dsl::*;

        transactions_addresses
            .inner_join(tx_dsl::transactions)
            .filter(tx_dsl::created_at.ge(since))
            .select((address, tx_dsl::chain))
            .group_by((address, tx_dsl::chain))
            .having(count(address).gt(min_count))
            .order_by(count(address).desc())
            .limit(limit)
            .load::<AddressChainIdResultRow>(&mut self.connection)
    }

    fn delete_transactions_addresses(&mut self, chain_addresses: Vec<AddressChainIdResultRow>) -> Result<Vec<i64>, diesel::result::Error> {
        use crate::schema::transactions::dsl as tx_dsl;
        use crate::schema::transactions_addresses::dsl as addr_dsl;

        if chain_addresses.is_empty() {
            return Ok(vec![]);
        }

        self.connection.transaction(|connection| {
            let mut transaction_ids = vec![];
            for row in chain_addresses {
                let mut deleted_ids = diesel::delete(addr_dsl::transactions_addresses.filter(addr_dsl::address.eq(row.address)).filter(exists(
                    tx_dsl::transactions.filter(tx_dsl::id.eq(addr_dsl::transaction_id)).filter(tx_dsl::chain.eq(row.chain_id)),
                )))
                .returning(addr_dsl::transaction_id)
                .load(connection)?;
                transaction_ids.append(&mut deleted_ids);
            }
            Ok(transaction_ids)
        })
    }

    fn delete_orphaned_transactions(&mut self, candidate_ids: Vec<i64>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        use crate::schema::transactions_addresses::dsl as addr;

        if candidate_ids.is_empty() {
            return Ok(0);
        }

        let ids: Vec<i64> = transactions
            .filter(id.eq_any(&candidate_ids))
            .left_outer_join(addr::transactions_addresses.on(id.eq(addr::transaction_id)))
            .filter(addr::transaction_id.is_null())
            .select(id)
            .load(&mut self.connection)?;

        if ids.is_empty() {
            return Ok(0);
        }

        diesel::delete(transactions.filter(id.eq_any(ids))).execute(&mut self.connection)
    }

    fn get_asset_usage_counts(&mut self, since: NaiveDateTime) -> Result<Vec<(AssetId, i64)>, diesel::result::Error> {
        use crate::schema::assets_addresses::dsl::*;

        assets_addresses
            .filter(updated_at.ge(since))
            .group_by(asset_id)
            .select((asset_id, count(asset_id)))
            .load::<(AssetId, i64)>(&mut self.connection)
    }

    fn get_transactions_by_wallet_since(&mut self, wallet_id: i32, since: NaiveDateTime, filters: Vec<TransactionFilter>) -> Result<Vec<TransactionRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl as tx_dsl;
        use crate::schema::transactions_addresses::dsl as addr_dsl;
        use crate::schema::wallets_addresses::dsl as wallet_addr_dsl;
        use crate::schema::wallets_subscriptions::dsl as wallet_sub_dsl;

        let mut query = tx_dsl::transactions
            .inner_join(addr_dsl::transactions_addresses.on(tx_dsl::id.eq(addr_dsl::transaction_id)))
            .inner_join(wallet_addr_dsl::wallets_addresses.on(addr_dsl::address.eq(wallet_addr_dsl::address)))
            .inner_join(wallet_sub_dsl::wallets_subscriptions.on(wallet_addr_dsl::id.eq(wallet_sub_dsl::address_id)))
            .into_boxed()
            .filter(wallet_sub_dsl::wallet_id.eq(wallet_id))
            .filter(tx_dsl::created_at.ge(since));

        for filter in filters {
            match filter {
                TransactionFilter::States(states) => {
                    query = query.filter(tx_dsl::state.eq_any(states));
                }
                TransactionFilter::Kinds(kinds) => {
                    query = query.filter(tx_dsl::kind.eq_any(kinds));
                }
            }
        }

        query.distinct().select(TransactionRow::as_select()).load(&mut self.connection)
    }

    fn get_transactions_by_filter(&mut self, filters: Vec<TransactionFilter>, limit: i64) -> Result<Vec<TransactionRow>, diesel::result::Error> {
        use crate::schema::transactions::dsl;
        let mut query = dsl::transactions.into_boxed();

        for filter in filters {
            match filter {
                TransactionFilter::States(states) => {
                    query = query.filter(dsl::state.eq_any(states));
                }
                TransactionFilter::Kinds(kinds) => {
                    query = query.filter(dsl::kind.eq_any(kinds));
                }
            }
        }

        query
            .order(dsl::created_at.asc())
            .limit(limit)
            .select(TransactionRow::as_select())
            .load(&mut self.connection)
    }

    fn update_transaction(&mut self, chain: &str, hash: &str, updates: Vec<TransactionUpdate>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl;

        if updates.is_empty() {
            return Ok(0);
        }

        let target = dsl::transactions.filter(dsl::chain.eq(chain).and(dsl::hash.eq(hash)));
        let mut total = 0;

        for update in &updates {
            let updated = match update {
                TransactionUpdate::State(state) => diesel::update(target).set(dsl::state.eq(state)).execute(&mut self.connection)?,
                TransactionUpdate::Kind(kind) => diesel::update(target).set(dsl::kind.eq(kind)).execute(&mut self.connection)?,
                TransactionUpdate::Metadata(metadata) => diesel::update(target).set(dsl::metadata.eq(metadata)).execute(&mut self.connection)?,
            };
            total += updated;
        }

        Ok(total)
    }

    fn get_addresses_by_chain_and_kind(&mut self, chain: &str, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::transactions::dsl as tx_dsl;
        use crate::schema::transactions_addresses::dsl::*;

        transactions_addresses
            .inner_join(tx_dsl::transactions)
            .filter(tx_dsl::chain.eq(chain))
            .filter(tx_dsl::kind.eq_any(kinds))
            .filter(tx_dsl::state.eq(TransactionState::Confirmed))
            .filter(tx_dsl::created_at.ge(since))
            .select(address)
            .distinct()
            .load::<String>(&mut self.connection)
    }
}
