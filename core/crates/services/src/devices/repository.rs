use std::collections::{BTreeMap, BTreeSet, HashMap};

use async_trait::async_trait;
use primitives::{Chain, ChainAddress, Device, WalletId, WalletSource, WalletSubscription, WalletSubscriptionChains};
use storage::{Database, DatabaseError, DeviceRecord, DevicesRepository, FiatRepository, NewWallet, NftRepository, PriceAlertsRepository, RewardsRepository, TransactionsRepository, WalletRecord, WalletsRepository};

use super::DeviceWalletLookup;
use super::admin_device::AdminWalletOverview;

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn add_device(&self, device: Device) -> Result<Device, DatabaseError>;
    async fn get_device(&self, device_id: String) -> Result<Device, DatabaseError>;
    async fn update_device(&self, device: Device) -> Result<Device, DatabaseError>;
    async fn get_device_exists(&self, device_id: String) -> Result<bool, DatabaseError>;
    async fn get_device_record(&self, device_id: String) -> Result<Option<DeviceRecord>, DatabaseError>;
    async fn get_device_wallet(&self, device_id: String, wallet_id: String) -> Result<DeviceWalletLookup, DatabaseError>;
    async fn get_device_with_price_alert_count(&self, device_id: String) -> Result<(DeviceRecord, i64), DatabaseError>;
    async fn get_subscriptions(&self, device_row_id: i32) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError>;
    async fn get_device_subscriptions(&self, device_id: String) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError>;
    async fn get_wallet_subscriptions(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError>;
    async fn get_device_wallet_subscriptions(&self, device_id: String, wallet_id: String) -> Result<(WalletRecord, Vec<ChainAddress>), DatabaseError>;
    async fn get_wallet_overviews(&self, device_row_id: i32) -> Result<Vec<AdminWalletOverview>, DatabaseError>;
    async fn add_subscriptions(&self, device_row_id: i32, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, DatabaseError>;
    async fn delete_subscriptions(&self, device_row_id: i32, subscriptions: Vec<WalletSubscriptionChains>) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn add_device(&self, device: Device) -> Result<Device, DatabaseError> {
        self.database.run(move |client| client.add_device(device)).await
    }

    async fn get_device(&self, device_id: String) -> Result<Device, DatabaseError> {
        self.database.run(move |client| client.get_device(&device_id)).await
    }

    async fn update_device(&self, device: Device) -> Result<Device, DatabaseError> {
        self.database.run(move |client| client.update_device(device)).await
    }

    async fn get_device_exists(&self, device_id: String) -> Result<bool, DatabaseError> {
        self.database.run(move |client| client.get_device_exist(&device_id)).await
    }

    async fn get_device_record(&self, device_id: String) -> Result<Option<DeviceRecord>, DatabaseError> {
        self.database.run(move |client| optional_record(client.get_device_record(&device_id))).await
    }

    async fn get_device_wallet(&self, device_id: String, wallet_id: String) -> Result<DeviceWalletLookup, DatabaseError> {
        self.database
            .run(move |client| {
                let Some(device) = optional_record(client.get_device_record(&device_id))? else {
                    return Ok(DeviceWalletLookup::DeviceNotFound);
                };
                Ok(match client.get_wallet_by_device_and_identifier(device.id, &wallet_id) {
                    Ok(wallet) => DeviceWalletLookup::Found(device, wallet),
                    Err(error) if error.is_not_found() => DeviceWalletLookup::WalletNotFound,
                    Err(_) => DeviceWalletLookup::WalletUnavailable,
                })
            })
            .await
    }

    async fn get_device_with_price_alert_count(&self, device_id: String) -> Result<(DeviceRecord, i64), DatabaseError> {
        self.database
            .run(move |client| {
                let device = client.get_device_record(&device_id)?;
                let price_alert_count = client.get_price_alerts_count_for_device_id(device.id)?;
                Ok((device, price_alert_count))
            })
            .await
    }

    async fn get_subscriptions(&self, device_row_id: i32) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError> {
        self.database.run(move |client| client.get_subscriptions(device_row_id)).await
    }

    async fn get_device_subscriptions(&self, device_id: String) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError> {
        self.database
            .run(move |client| {
                let device_row_id = client.get_device_row_id(&device_id)?;
                client.get_subscriptions(device_row_id)
            })
            .await
    }

    async fn get_wallet_subscriptions(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError> {
        self.database.run(move |client| client.get_subscriptions_by_wallet_id(device_row_id, wallet_id)).await
    }

    async fn get_device_wallet_subscriptions(&self, device_id: String, wallet_id: String) -> Result<(WalletRecord, Vec<ChainAddress>), DatabaseError> {
        self.database
            .run(move |client| {
                let device_row_id = client.get_device_row_id(&device_id)?;
                let wallet = client.get_wallet_by_device_and_identifier(device_row_id, &wallet_id)?;
                let rows = client.get_subscriptions_by_wallet_id(device_row_id, wallet.id)?;
                Ok((wallet, rows))
            })
            .await
    }

    async fn get_wallet_overviews(&self, device_row_id: i32) -> Result<Vec<AdminWalletOverview>, DatabaseError> {
        self.database
            .run(move |client| {
                let rows = client.get_subscriptions(device_row_id)?;
                rows.into_iter()
                    .fold(BTreeMap::<String, WalletOverviewBuilder>::new(), |mut wallets, (wallet, subscription)| {
                        let entry = wallets.entry(wallet.wallet_id.id()).or_insert_with(|| WalletOverviewBuilder {
                            wallet_id: wallet.id,
                            identifier: wallet.wallet_id,
                            source: wallet.source,
                            addresses: BTreeSet::new(),
                            chains: BTreeSet::new(),
                            subscription_count: 0,
                        });
                        entry.subscription_count += 1;
                        entry.addresses.insert(subscription.address);
                        entry.chains.insert(subscription.chain);
                        wallets
                    })
                    .into_values()
                    .map(|wallet| {
                        let chains = wallet.chains.into_iter().collect::<Vec<_>>();
                        let addresses = wallet.addresses.into_iter().collect::<Vec<_>>();
                        Ok(AdminWalletOverview {
                            transaction_count: client.get_transactions_count_by_addresses(addresses.clone(), chains.iter().map(|chain| chain.as_ref().to_string()).collect())?,
                            fiat_transaction_count: client.get_fiat_transactions_count_by_device_and_wallet_id(device_row_id, wallet.wallet_id)?,
                            nft_count: client.get_nft_assets_count_by_addresses(addresses, chains.clone())?,
                            chains,
                            id: wallet.identifier,
                            source: wallet.source,
                            username: client.get_username_by_wallet_id(wallet.wallet_id)?,
                            subscription_count: wallet.subscription_count,
                        })
                    })
                    .collect()
            })
            .await
    }

    async fn add_subscriptions(&self, device_row_id: i32, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
                let mut wallet_ids: HashMap<String, i32> = client.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

                let new_wallets: Vec<NewWallet> = wallet_subscriptions
                    .iter()
                    .filter(|x| !wallet_ids.contains_key(&x.wallet_id.id()))
                    .map(|x| NewWallet {
                        wallet_id: x.wallet_id.clone(),
                        wallet_type: x.wallet_id.wallet_type(),
                        source: x.source.clone().unwrap_or(WalletSource::Import),
                    })
                    .collect();

                if !new_wallets.is_empty() {
                    let new_identifiers: Vec<String> = new_wallets.iter().map(|x| x.wallet_id.id()).collect();
                    client.add_wallets(new_wallets)?;
                    wallet_ids.extend(client.get_wallets(new_identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)));
                }

                let subscriptions: Vec<(i32, Chain, String)> = wallet_subscriptions
                    .iter()
                    .filter_map(|ws| wallet_ids.get(&ws.wallet_id.id()).map(|&wallet_id| ws.chain_addresses().into_iter().map(move |ca| (wallet_id, ca.chain, ca.address))))
                    .flatten()
                    .collect();

                client.add_subscriptions(device_row_id, subscriptions)
            })
            .await
    }

    async fn delete_subscriptions(&self, device_row_id: i32, subscriptions: Vec<WalletSubscriptionChains>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                let identifiers: Vec<String> = subscriptions.iter().map(|x| x.wallet_id.id()).collect();
                let wallet_ids: HashMap<String, i32> = client.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();
                let mut count = 0;
                for ws in subscriptions {
                    if let Some(&wallet_id) = wallet_ids.get(&ws.wallet_id.id()) {
                        count += client.delete_wallet_chains(device_row_id, wallet_id, ws.chains)?;
                    }
                }
                Ok(count)
            })
            .await
    }
}

struct WalletOverviewBuilder {
    wallet_id: i32,
    identifier: WalletId,
    source: WalletSource,
    addresses: BTreeSet<String>,
    chains: BTreeSet<Chain>,
    subscription_count: usize,
}

fn optional_record<T>(result: Result<T, DatabaseError>) -> Result<Option<T>, DatabaseError> {
    match result {
        Ok(record) => Ok(Some(record)),
        Err(error) if error.is_not_found() => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use storage::DatabaseError;

    use super::optional_record;

    #[test]
    fn test_optional_record() {
        assert_eq!(optional_record(Ok(7)).unwrap(), Some(7));
        assert_eq!(optional_record::<i32>(Err(DatabaseError::not_found("Device", "device_1"))).unwrap(), None);
        assert!(matches!(optional_record::<i32>(Err(DatabaseError::ConnectionPool)), Err(DatabaseError::ConnectionPool)));
        assert!(matches!(optional_record::<i32>(Err(DatabaseError::Error("timeout".to_string()))), Err(DatabaseError::Error(_))));
    }
}
