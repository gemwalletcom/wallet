use std::error::Error;
use std::sync::Arc;

use cacher::AccessTokenCacherClient;
use config_keys::ConfigKey;
use gem_client::ReqwestClient;
use primitives::{Chain, PriceProvider};
use rewards::TransferRedemptionService;
use security::providers::goplus::GoPlusProvider;
use security::{ScanProviderConfig, ScanProviderFactory, TokenScanProviders};
use streamer::{ShutdownReceiver, StreamProducer};

use crate::Services;
use crate::assets::{AssetClassificationRules, FetchAssetAssociationsConsumer, FetchAssetStatusConsumer, FetchAssetsConsumer, FetchCoinAddressesConsumer, FetchListConsumer, FetchTokenAddressesConsumer};
use crate::fiat::FiatWebhookConsumer;
use crate::nft::{FetchNftAssetConsumer, FetchNftAssetsAddressesConsumer};
use crate::notifications::{InAppNotificationsConsumer, NotificationsConsumer, NotificationsFailedConsumer, Pusher};
use crate::prices::{FetchPricesConsumer, FetchPricesMetadataConsumer, StorePricesConsumer};
use crate::rewards::{RedemptionRetryConfig, RewardsConsumer, RewardsRedemptionConsumer};
use crate::support::SupportWebhookConsumer;
use crate::transactions::{
    FetchAddressTransactionsConsumer, FetchBlocksConsumer, FetchTransactionConsumer, StorePendingTransactionsConsumer, StoreTransactionsConsumer, StoreTransactionsPerpetualsConsumer, StoreTransactionsSwapsConsumer, SwapVaultAddressClient,
    WalletStreamConsumer,
};

impl Services {
    pub fn fetch_asset_associations_consumer(&self) -> FetchAssetAssociationsConsumer {
        FetchAssetAssociationsConsumer {
            repository: self.assets_repository(),
            providers: self.price_providers(PriceProvider::all()),
        }
    }

    pub fn fetch_blocks_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> FetchBlocksConsumer {
        FetchBlocksConsumer::new(self.chain_providers_for(chain, user_agent), Arc::new(stream_producer))
    }

    pub async fn fetch_assets_consumer(&self, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchAssetsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchAssetsConsumer {
            repository: self.assets_repository(),
            providers: self.chain_providers(user_agent),
            throttle: Arc::new(self.cacher().await?),
            classification_rules: AssetClassificationRules::from_config(&self.config()).await?,
            stream_producer: Arc::new(stream_producer),
        })
    }

    pub async fn fetch_asset_status_consumer(&self) -> Result<FetchAssetStatusConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchAssetStatusConsumer {
            repository: self.assets_repository(),
            providers: self.token_scan_providers().await?,
        })
    }

    pub fn fetch_list_consumer(&self) -> FetchListConsumer {
        FetchListConsumer { lists_client: self.lists() }
    }

    pub async fn fetch_prices_consumer(&self) -> Result<FetchPricesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchPricesConsumer {
            price_client: self.prices(self.cacher().await?),
            providers: self.price_providers(PriceProvider::all()),
        })
    }

    pub async fn fetch_prices_metadata_consumer(&self) -> Result<FetchPricesMetadataConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchPricesMetadataConsumer {
            repository: self.prices_repository(),
            cooldowns: Arc::new(self.cacher().await?),
            config: self.config(),
            providers: self.price_providers(PriceProvider::all()),
        })
    }

    pub async fn fetch_token_addresses_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchTokenAddressesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchTokenAddressesConsumer::new(
            self.chain_providers_for(chain, user_agent),
            self.assets_repository(),
            Arc::new(stream_producer),
            Arc::new(self.cacher().await?),
        ))
    }

    pub async fn fetch_coin_addresses_consumer(&self, chain: Chain, user_agent: &str) -> Result<FetchCoinAddressesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchCoinAddressesConsumer::new(self.chain_providers_for(chain, user_agent), self.assets_repository(), Arc::new(self.cacher().await?)))
    }

    pub async fn fetch_nft_asset_consumer(&self) -> Result<FetchNftAssetConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchNftAssetConsumer {
            nft_client: self.nft(),
            throttle: Arc::new(self.cacher().await?),
        })
    }

    pub async fn fetch_nft_assets_addresses_consumer(&self) -> Result<FetchNftAssetsAddressesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchNftAssetsAddressesConsumer {
            throttle: Arc::new(self.cacher().await?),
            nft_client: self.nft(),
        })
    }

    pub async fn fetch_address_transactions_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchAddressTransactionsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchAddressTransactionsConsumer::new(
            self.chain_providers_for(chain, user_agent),
            Arc::new(stream_producer),
            Arc::new(self.cacher().await?),
            self.config(),
        ))
    }

    pub async fn fetch_transaction_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchTransactionConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchTransactionConsumer::new(
            self.chain_providers_for(chain, user_agent),
            self.swapper(),
            Arc::new(stream_producer),
            Arc::new(self.cacher().await?),
            self.transactions_repository(),
        ))
    }

    pub async fn store_transactions_consumer(&self, stream_producer: StreamProducer) -> Result<StoreTransactionsConsumer, Box<dyn Error + Send + Sync>> {
        let cacher = self.cacher().await?;
        Ok(StoreTransactionsConsumer {
            repository: self.transactions_repository(),
            stream_producer: Arc::new(stream_producer),
            pusher: Pusher::new(self.notifications_repository()),
            config: self.config(),
            vault_client: SwapVaultAddressClient::new(Arc::new(cacher.clone())),
            subscription_lookup: self.subscription_lookup(cacher),
        })
    }

    pub async fn store_prices_consumer(&self) -> Result<StorePricesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(StorePricesConsumer::new(self.prices_repository(), self.prices(self.cacher().await?), self.config()))
    }

    pub async fn wallet_stream_consumer(&self) -> Result<WalletStreamConsumer, Box<dyn Error + Send + Sync>> {
        Ok(WalletStreamConsumer {
            repository: self.transactions_repository(),
            device_stream: Arc::new(self.cacher().await?),
            retention: self.config().get_duration(ConfigKey::DeviceStreamRetention).await?,
        })
    }

    pub async fn store_pending_transactions_consumer(&self) -> Result<StorePendingTransactionsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(StorePendingTransactionsConsumer::new(Arc::new(self.cacher().await?)))
    }

    pub fn store_transactions_swaps_consumer(&self) -> StoreTransactionsSwapsConsumer {
        StoreTransactionsSwapsConsumer::new(self.transactions_repository(), self.config())
    }

    pub fn store_transactions_perpetuals_consumer(&self) -> StoreTransactionsPerpetualsConsumer {
        StoreTransactionsPerpetualsConsumer::new(self.transactions_repository())
    }

    pub async fn notifications_consumer(&self, name: &str, shutdown: ShutdownReceiver) -> Result<NotificationsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(NotificationsConsumer::new(self.pusher(), Arc::new(self.stream_producer(name, shutdown).await?)))
    }

    pub fn notifications_failed_consumer(&self) -> NotificationsFailedConsumer {
        NotificationsFailedConsumer::new(self.notifications_repository())
    }

    pub async fn in_app_notifications_consumer(&self, name: &str, shutdown: ShutdownReceiver) -> Result<InAppNotificationsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(InAppNotificationsConsumer::new(self.notifications_repository(), Arc::new(self.stream_producer(name, shutdown).await?)))
    }

    pub async fn rewards_consumer(&self, name: &str, shutdown: ShutdownReceiver) -> Result<RewardsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(RewardsConsumer::new(self.rewards_repository(), Arc::new(self.stream_producer(name, shutdown).await?)))
    }

    pub async fn rewards_redemption_consumer(&self, name: &str, shutdown: ShutdownReceiver) -> Result<RewardsRedemptionConsumer<TransferRedemptionService>, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        let retry_config = RedemptionRetryConfig {
            max_retries: config.get_i64(ConfigKey::RedemptionRetryMaxRetries).await? as u32,
            delay: config.get_duration(ConfigKey::RedemptionRetryDelay).await?,
            errors: config.get_json(ConfigKey::RedemptionRetryErrors).await?,
        };
        let stream_producer = self.stream_producer(name, shutdown).await?;
        Ok(RewardsRedemptionConsumer::new(self.rewards_repository(), Arc::new(self.redemption_service()?), retry_config, Arc::new(stream_producer)))
    }

    pub async fn fiat_webhook_consumer(&self, name: &str, shutdown: ShutdownReceiver) -> Result<FiatWebhookConsumer, Box<dyn Error + Send + Sync>> {
        let stream_producer = self.stream_producer(&format!("{name}_producer"), shutdown).await?;
        let providers = self.fiat_providers(self.fiat_access_token_cacher().await?);
        Ok(FiatWebhookConsumer::new(self.fiat_repository(), providers, Arc::new(stream_producer)))
    }

    pub async fn support_webhook_consumer(&self, shutdown: ShutdownReceiver) -> Result<SupportWebhookConsumer, Box<dyn Error + Send + Sync>> {
        Ok(SupportWebhookConsumer::new(self.support(shutdown).await?))
    }

    async fn token_scan_providers(&self) -> Result<TokenScanProviders, Box<dyn Error + Send + Sync>> {
        let config = ScanProviderConfig::new(&self.settings().security, self.config().get_duration(ConfigKey::ScanTimeout).await?);
        let access_tokens = Arc::new(AccessTokenCacherClient::new(self.cacher().await?, GoPlusProvider::<ReqwestClient>::NAME));
        ScanProviderFactory::new_token_providers(&config, access_tokens)
    }
}
