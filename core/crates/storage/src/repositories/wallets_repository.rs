use std::collections::{HashMap, HashSet};

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{Chain, ChainAddress, Device, DeviceSubscription, WalletId, WalletSource, WalletType};

use crate::models::{DeviceRow, NewWalletAddressRow, NewWalletRow, NewWalletSubscriptionRow, SubscriptionAddressExcludeRow, WalletAddressRow, WalletRow, WalletSubscriptionRow};
use crate::schema::{devices, subscriptions_addresses_exclude, wallets, wallets_addresses, wallets_subscriptions};
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone, PartialEq)]
pub struct WalletRecord {
    pub id: i32,
    pub wallet_id: WalletId,
    pub wallet_type: WalletType,
    pub source: WalletSource,
}

impl WalletRecord {
    pub(crate) fn from_row(row: WalletRow) -> Self {
        Self {
            id: row.id,
            wallet_id: row.wallet_id.0,
            wallet_type: row.wallet_type.0,
            source: row.source.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewWallet {
    pub wallet_id: WalletId,
    pub wallet_type: WalletType,
    pub source: WalletSource,
}

impl NewWallet {
    fn as_row(&self) -> NewWalletRow {
        NewWalletRow {
            identifier: self.wallet_id.id(),
            wallet_type: self.wallet_type.into(),
            source: self.source.clone().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletAddress {
    pub id: i32,
    pub address: String,
}

pub trait WalletsRepository {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRecord, DatabaseError>;
    fn get_wallet_by_device_and_identifier(&mut self, device_id: i32, identifier: &str) -> Result<WalletRecord, DatabaseError>;
    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRecord, DatabaseError>;
    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRecord>, DatabaseError>;
    fn get_device_multicoin_wallet_ids(&mut self, device_id: i32, chain: Chain) -> Result<Vec<i32>, DatabaseError>;
    fn create_wallets(&mut self, wallets: Vec<NewWallet>) -> Result<usize, DatabaseError>;
    fn get_or_create_wallet(&mut self, wallet: NewWallet) -> Result<WalletRecord, DatabaseError>;
    fn get_subscriptions(&mut self, device_id: i32) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError>;
    fn get_subscriptions_by_wallet_id(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError>;
    fn subscriptions_wallet_address_for_chain(&mut self, device_id: i32, wallet_id: i32, chain: Chain) -> Result<WalletAddress, DatabaseError>;
    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<Device>, DatabaseError>;
    fn add_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError>;
    fn delete_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError>;
    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, DatabaseError>;

    fn get_subscriptions_by_chain_addresses(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError>;
    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<ChainAddress>) -> Result<usize, DatabaseError>;
}

pub(crate) fn wallet_row(client: &mut DatabaseClient, identifier: &str) -> Result<WalletRow, diesel::result::Error> {
    wallets::table.filter(wallets::identifier.eq(identifier)).select(WalletRow::as_select()).first(&mut client.connection)
}

fn wallet_addresses(client: &mut DatabaseClient, addresses: Vec<String>) -> Result<Vec<WalletAddressRow>, diesel::result::Error> {
    wallets_addresses::table.filter(wallets_addresses::address.eq_any(addresses)).select(WalletAddressRow::as_select()).load(&mut client.connection)
}

pub(crate) fn wallet_row_by_id(client: &mut DatabaseClient, id: i32) -> Result<WalletRow, DatabaseError> {
    wallets::table.filter(wallets::id.eq(id)).select(WalletRow::as_select()).first(&mut client.connection).or_not_found_internal(id.to_string())
}

pub(crate) fn device_rows_by_wallet_id(client: &mut DatabaseClient, wallet_id: i32) -> Result<Vec<DeviceRow>, diesel::result::Error> {
    wallets_subscriptions::table
        .inner_join(devices::table)
        .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
        .select(DeviceRow::as_select())
        .distinct()
        .load(&mut client.connection)
}

pub(crate) fn delete_subscriptions_for_device_ids(client: &mut DatabaseClient, device_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
    diesel::delete(wallets_subscriptions::table).filter(wallets_subscriptions::device_id.eq_any(device_ids)).execute(&mut client.connection)
}

pub(crate) fn device_addresses(client: &mut DatabaseClient, device_id: i32, chain: ChainRow) -> Result<Vec<String>, diesel::result::Error> {
    wallets_subscriptions::table
        .inner_join(wallets_addresses::table)
        .filter(wallets_subscriptions::device_id.eq(device_id))
        .filter(wallets_subscriptions::chain.eq(chain))
        .select(wallets_addresses::address)
        .load(&mut client.connection)
}

pub(crate) fn first_subscription_date_by_wallet_id(client: &mut DatabaseClient, wallet_id: i32) -> Result<Option<NaiveDateTime>, diesel::result::Error> {
    wallets_subscriptions::table
        .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
        .select(wallets_subscriptions::created_at)
        .order(wallets_subscriptions::created_at.asc())
        .first(&mut client.connection)
        .optional()
}

fn delete_address_subscriptions(client: &mut DatabaseClient, device_id: i32, wallet_id: i32, chain: ChainRow, address_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
    diesel::delete(wallets_subscriptions::table)
        .filter(wallets_subscriptions::device_id.eq(device_id))
        .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
        .filter(wallets_subscriptions::chain.eq(chain))
        .filter(wallets_subscriptions::address_id.eq_any(address_ids))
        .execute(&mut client.connection)
}

impl WalletsRepository for DatabaseClient {
    fn get_wallet(&mut self, identifier: &str) -> Result<WalletRecord, DatabaseError> {
        Ok(WalletRecord::from_row(wallet_row(self, identifier).or_not_found(identifier.to_string())?))
    }

    fn get_wallet_by_device_and_identifier(&mut self, device_id: i32, identifier: &str) -> Result<WalletRecord, DatabaseError> {
        let row = wallets::table
            .inner_join(wallets_subscriptions::table.on(wallets_subscriptions::wallet_id.eq(wallets::id)))
            .filter(wallets::identifier.eq(identifier))
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .select(WalletRow::as_select())
            .first(&mut self.connection)
            .or_not_found(identifier.to_string())?;
        Ok(WalletRecord::from_row(row))
    }

    fn get_wallet_by_id(&mut self, id: i32) -> Result<WalletRecord, DatabaseError> {
        Ok(WalletRecord::from_row(wallet_row_by_id(self, id)?))
    }

    fn get_device_multicoin_wallet_ids(&mut self, device_id: i32, chain: Chain) -> Result<Vec<i32>, DatabaseError> {
        let mut wallet_ids = Vec::new();
        for address in device_addresses(self, device_id, ChainRow::from(chain))? {
            match wallet_row(self, &WalletId::Multicoin(address).id()) {
                Ok(wallet) => wallet_ids.push(wallet.id),
                Err(diesel::result::Error::NotFound) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(wallet_ids)
    }

    fn get_wallets(&mut self, identifiers: Vec<String>) -> Result<Vec<WalletRecord>, DatabaseError> {
        let rows = wallets::table.filter(wallets::identifier.eq_any(identifiers)).select(WalletRow::as_select()).load(&mut self.connection)?;
        Ok(rows.into_iter().map(WalletRecord::from_row).collect())
    }

    fn create_wallets(&mut self, new_wallets: Vec<NewWallet>) -> Result<usize, DatabaseError> {
        let rows: Vec<NewWalletRow> = new_wallets.iter().map(NewWallet::as_row).collect();
        Ok(diesel::insert_into(wallets::table).values(&rows).on_conflict(wallets::identifier).do_nothing().execute(&mut self.connection)?)
    }

    fn get_or_create_wallet(&mut self, wallet: NewWallet) -> Result<WalletRecord, DatabaseError> {
        let wallet = wallet.as_row();
        let row = match wallet_row(self, &wallet.identifier) {
            Ok(existing) => existing,
            Err(diesel::result::Error::NotFound) => diesel::insert_into(wallets::table).values(&wallet).returning(WalletRow::as_returning()).get_result(&mut self.connection)?,
            Err(error) => return Err(error.into()),
        };
        Ok(WalletRecord::from_row(row))
    }

    fn get_subscriptions(&mut self, device_id: i32) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError> {
        let rows: Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow)> = wallets_subscriptions::table
            .inner_join(wallets::table)
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .select((WalletRow::as_select(), WalletSubscriptionRow::as_select(), WalletAddressRow::as_select()))
            .load(&mut self.connection)?;
        Ok(rows
            .into_iter()
            .map(|(wallet, subscription, address)| (WalletRecord::from_row(wallet), ChainAddress::new(subscription.chain.0, address.address)))
            .collect())
    }

    fn get_subscriptions_by_wallet_id(&mut self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError> {
        let rows: Vec<(WalletSubscriptionRow, WalletAddressRow)> = wallets_subscriptions::table
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .select((WalletSubscriptionRow::as_select(), WalletAddressRow::as_select()))
            .load(&mut self.connection)?;
        Ok(rows.into_iter().map(|(subscription, address)| ChainAddress::new(subscription.chain.0, address.address)).collect())
    }

    fn subscriptions_wallet_address_for_chain(&mut self, device_id: i32, wallet_id: i32, chain: Chain) -> Result<WalletAddress, DatabaseError> {
        let row = wallets_subscriptions::table
            .inner_join(wallets_addresses::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq(ChainRow::from(chain)))
            .select(WalletAddressRow::as_select())
            .first(&mut self.connection)
            .or_not_found_for::<WalletAddressRow>(chain.as_ref().to_string())?;
        Ok(WalletAddress { id: row.id, address: row.address })
    }

    fn get_devices_by_wallet_id(&mut self, wallet_id: i32) -> Result<Vec<Device>, DatabaseError> {
        Ok(device_rows_by_wallet_id(self, wallet_id)?.iter().map(DeviceRow::as_primitive).collect())
    }

    fn add_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        let all_addresses: Vec<String> = subscriptions.iter().map(|(_, _, addr)| addr.clone()).collect::<HashSet<_>>().into_iter().collect();

        let existing_rows = wallet_addresses(self, all_addresses.clone())?;
        let existing_set: HashSet<String> = existing_rows.iter().map(|row| row.address.clone()).collect();

        let missing_addresses: Vec<NewWalletAddressRow> = all_addresses.iter().filter(|addr| !existing_set.contains(*addr)).map(|address| NewWalletAddressRow { address: address.clone() }).collect();

        let new_rows = if missing_addresses.is_empty() {
            vec![]
        } else {
            let missing_strs: Vec<String> = missing_addresses.iter().map(|a| a.address.clone()).collect();
            diesel::insert_into(wallets_addresses::table)
                .values(&missing_addresses)
                .on_conflict(wallets_addresses::address)
                .do_nothing()
                .execute(&mut self.connection)?;
            wallet_addresses(self, missing_strs)?
        };

        let address_map: HashMap<String, i32> = existing_rows.into_iter().chain(new_rows).map(|row| (row.address, row.id)).collect();

        let rows: Vec<NewWalletSubscriptionRow> = subscriptions
            .into_iter()
            .filter_map(|(wallet_id, chain, address)| {
                address_map.get(&address).map(|&address_id| NewWalletSubscriptionRow {
                    wallet_id,
                    device_id,
                    chain: ChainRow::from(chain),
                    address_id,
                })
            })
            .collect();

        if rows.is_empty() {
            return Ok(0);
        }

        Ok(diesel::insert_into(wallets_subscriptions::table)
            .values(&rows)
            .on_conflict((wallets_subscriptions::wallet_id, wallets_subscriptions::device_id, wallets_subscriptions::chain, wallets_subscriptions::address_id))
            .do_nothing()
            .execute(&mut self.connection)?)
    }

    fn delete_subscriptions(&mut self, device_id: i32, subscriptions: Vec<(i32, Chain, String)>) -> Result<usize, DatabaseError> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        let all_addresses: Vec<String> = subscriptions.iter().map(|(_, _, addr)| addr.clone()).collect::<HashSet<_>>().into_iter().collect();

        let address_rows = wallet_addresses(self, all_addresses)?;
        let address_map: HashMap<String, i32> = address_rows.into_iter().map(|row| (row.address, row.id)).collect();

        let mut grouped: HashMap<(i32, Chain), Vec<i32>> = HashMap::new();
        for (wallet_id, chain, address) in subscriptions {
            if let Some(&address_id) = address_map.get(&address) {
                grouped.entry((wallet_id, chain)).or_default().push(address_id);
            }
        }

        if grouped.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        for ((wallet_id, chain), address_ids) in grouped {
            count += delete_address_subscriptions(self, device_id, wallet_id, ChainRow::from(chain), address_ids)?;
        }

        Ok(count)
    }

    fn delete_wallet_chains(&mut self, device_id: i32, wallet_id: i32, chains: Vec<Chain>) -> Result<usize, DatabaseError> {
        let chain_rows: Vec<ChainRow> = chains.into_iter().map(ChainRow::from).collect();

        Ok(diesel::delete(wallets_subscriptions::table)
            .filter(wallets_subscriptions::device_id.eq(device_id))
            .filter(wallets_subscriptions::wallet_id.eq(wallet_id))
            .filter(wallets_subscriptions::chain.eq_any(chain_rows))
            .execute(&mut self.connection)?)
    }

    fn get_subscriptions_by_chain_addresses(&mut self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError> {
        let chain_row = ChainRow::from(chain);

        let rows: Vec<(WalletRow, WalletSubscriptionRow, WalletAddressRow, DeviceRow)> = wallets_subscriptions::table
            .inner_join(wallets::table)
            .inner_join(wallets_addresses::table)
            .inner_join(devices::table)
            .filter(wallets_subscriptions::chain.eq(chain_row))
            .filter(wallets_addresses::address.eq_any(&addresses))
            .filter(diesel::dsl::not(diesel::dsl::exists(
                subscriptions_addresses_exclude::table.filter(subscriptions_addresses_exclude::address.eq(wallets_addresses::address)),
            )))
            .select((WalletRow::as_select(), WalletSubscriptionRow::as_select(), WalletAddressRow::as_select(), DeviceRow::as_select()))
            .load(&mut self.connection)?;
        Ok(rows
            .into_iter()
            .map(|(wallet, sub, addr, device)| DeviceSubscription {
                wallet_row_id: wallet.id,
                device: device.as_primitive(),
                wallet_id: wallet.wallet_id.0.clone(),
                chain: sub.chain.0,
                address: addr.address,
            })
            .collect())
    }

    fn add_subscriptions_exclude_addresses(&mut self, values: Vec<ChainAddress>) -> Result<usize, DatabaseError> {
        let rows: Vec<SubscriptionAddressExcludeRow> = values
            .into_iter()
            .map(|value| SubscriptionAddressExcludeRow {
                address: value.address,
                chain: value.chain.into(),
            })
            .collect();
        Ok(diesel::insert_into(subscriptions_addresses_exclude::table).values(rows).on_conflict_do_nothing().execute(&mut self.connection)?)
    }
}
