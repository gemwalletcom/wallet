use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum QueueName {
    StoreTransactions,
    NotificationsPriceAlerts,
    NotificationsTransactions,
    NotificationsObservers,
    NotificationsSupport,
    NotificationsRewards,
    NotificationsFailed,
    FetchAssets,
    FetchAssetStatus,
    FetchAssetAssociations,
    FetchPrices,
    FetchPricesMetadata,
    FetchLists,
    FetchBlocks,
    FetchNFTCollectionAssets,
    FetchTokenAssociations,
    FetchCoinAssociations,
    FetchNftAssociations,
    FetchAddressTransactions,
    FetchTransactions,
    FiatOrderWebhooks,
    SupportWebhooks,
    StorePendingTransactions,
    StoreTransactionsSwaps,
    StorePrices,
    RewardsEvents,
    RewardsRedemptions,
    NotificationsFiatPurchase,
    NotificationsInApp,
    WalletStreamEvents,
}

impl QueueName {
    pub fn all() -> Vec<QueueName> {
        QueueName::iter().collect()
    }

    pub fn chain_queues() -> Vec<QueueName> {
        vec![
            QueueName::StoreTransactions,
            QueueName::FetchBlocks,
            QueueName::FetchTokenAssociations,
            QueueName::FetchCoinAssociations,
            QueueName::FetchNftAssociations,
            QueueName::FetchAddressTransactions,
            QueueName::FetchTransactions,
        ]
    }
}

impl fmt::Display for QueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueName::StoreTransactions => write!(f, "store_transactions"),
            QueueName::NotificationsPriceAlerts => write!(f, "notifications_price_alerts"),
            QueueName::NotificationsTransactions => write!(f, "notifications_transactions"),
            QueueName::NotificationsObservers => write!(f, "notifications_observers"),
            QueueName::FetchAssets => write!(f, "fetch_assets"),
            QueueName::FetchAssetStatus => write!(f, "fetch_asset_status"),
            QueueName::FetchAssetAssociations => write!(f, "fetch_asset_associations"),
            QueueName::FetchPrices => write!(f, "fetch_prices"),
            QueueName::FetchPricesMetadata => write!(f, "fetch_prices_metadata"),
            QueueName::FetchLists => write!(f, "fetch_lists"),
            QueueName::FetchBlocks => write!(f, "fetch_blocks"),
            QueueName::FetchNFTCollectionAssets => write!(f, "fetch_nft_collection_assets"),
            QueueName::FetchTokenAssociations => write!(f, "fetch_token_associations"),
            QueueName::FetchCoinAssociations => write!(f, "fetch_coin_associations"),
            QueueName::FetchAddressTransactions => write!(f, "fetch_address_transactions"),
            QueueName::FetchTransactions => write!(f, "fetch_transactions"),
            QueueName::FetchNftAssociations => write!(f, "fetch_nft_associations"),
            QueueName::FiatOrderWebhooks => write!(f, "fiat_order_webhooks"),
            QueueName::SupportWebhooks => write!(f, "support_webhooks"),
            QueueName::StorePendingTransactions => write!(f, "store_pending_transactions"),
            QueueName::StoreTransactionsSwaps => write!(f, "store_transactions_swaps"),
            QueueName::NotificationsSupport => write!(f, "notifications_support"),
            QueueName::NotificationsRewards => write!(f, "notifications_rewards"),
            QueueName::RewardsEvents => write!(f, "rewards_events"),
            QueueName::RewardsRedemptions => write!(f, "rewards_redemptions"),
            QueueName::NotificationsFailed => write!(f, "notifications_failed"),
            QueueName::StorePrices => write!(f, "store_prices"),
            QueueName::NotificationsFiatPurchase => write!(f, "notifications_fiat_purchase"),
            QueueName::NotificationsInApp => write!(f, "notifications_in_app"),
            QueueName::WalletStreamEvents => write!(f, "wallet_stream_events"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::QueueName;

    #[test]
    fn test_fetch_transactions_queue() {
        assert_eq!(QueueName::FetchTransactions.to_string(), "fetch_transactions");
        assert_eq!(QueueName::chain_queues().into_iter().find(|queue| queue == &QueueName::FetchTransactions), Some(QueueName::FetchTransactions));
    }
}
