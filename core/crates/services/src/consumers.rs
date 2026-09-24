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
use crate::transactions::{FetchAddressTransactionsConsumer, FetchBlocksConsumer, FetchTransactionConsumer, StorePendingTransactionsConsumer, StoreTransactionsConsumer, SwapVaultAddressClient, WalletStreamConsumer};

impl Services {
    pub fn fetch_asset_associations_consumer(&self) -> FetchAssetAssociationsConsumer {
        FetchAssetAssociationsConsumer {
            database: self.database(),
            providers: self.price_providers(PriceProvider::all()),
        }
    }

    pub fn fetch_blocks_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> FetchBlocksConsumer {
        FetchBlocksConsumer::new(self.chain_providers_for(chain, user_agent), stream_producer)
    }

    pub async fn fetch_assets_consumer(&self, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchAssetsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchAssetsConsumer {
            database: self.database(),
            providers: self.chain_providers(user_agent),
            cacher: self.cacher().await?,
            classification_rules: AssetClassificationRules::from_config(&self.config()).await?,
            stream_producer,
        })
    }

    pub async fn fetch_asset_status_consumer(&self) -> Result<FetchAssetStatusConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchAssetStatusConsumer {
            database: self.database(),
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
            database: self.database(),
            cacher: self.cacher().await?,
            config: self.config(),
            providers: self.price_providers(PriceProvider::all()),
        })
    }

    pub async fn fetch_token_addresses_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchTokenAddressesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchTokenAddressesConsumer::new(self.chain_providers_for(chain, user_agent), self.database(), stream_producer, self.cacher().await?))
    }

    pub async fn fetch_coin_addresses_consumer(&self, chain: Chain, user_agent: &str) -> Result<FetchCoinAddressesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchCoinAddressesConsumer::new(self.chain_providers_for(chain, user_agent), self.database(), self.cacher().await?))
    }

    pub async fn fetch_nft_asset_consumer(&self) -> Result<FetchNftAssetConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchNftAssetConsumer {
            nft_client: self.nft(),
            cacher: self.cacher().await?,
        })
    }

    pub async fn fetch_nft_assets_addresses_consumer(&self) -> Result<FetchNftAssetsAddressesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchNftAssetsAddressesConsumer {
            cacher: self.cacher().await?,
            nft_client: self.nft(),
        })
    }

    pub async fn fetch_address_transactions_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchAddressTransactionsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchAddressTransactionsConsumer::new(self.chain_providers_for(chain, user_agent), stream_producer, self.cacher().await?, self.config()))
    }

    pub async fn fetch_transaction_consumer(&self, chain: Chain, user_agent: &str, stream_producer: StreamProducer) -> Result<FetchTransactionConsumer, Box<dyn Error + Send + Sync>> {
        Ok(FetchTransactionConsumer::new(self.chain_providers_for(chain, user_agent), stream_producer, self.cacher().await?))
    }

    pub async fn store_transactions_consumer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<StoreTransactionsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(StoreTransactionsConsumer {
            database: self.database(),
            stream_producer: self.stream_producer(name, shutdown_rx).await?,
            pusher: Pusher::new(self.database()),
            config: self.config(),
            vault_client: SwapVaultAddressClient::new(self.cacher().await?),
        })
    }

    pub async fn store_prices_consumer(&self) -> Result<StorePricesConsumer, Box<dyn Error + Send + Sync>> {
        Ok(StorePricesConsumer::new(self.database(), self.prices(self.cacher().await?), self.config()))
    }

    pub async fn wallet_stream_consumer(&self) -> Result<WalletStreamConsumer, Box<dyn Error + Send + Sync>> {
        Ok(WalletStreamConsumer {
            database: self.database(),
            cacher_client: self.cacher().await?,
            retention: self.config().get_duration(ConfigKey::DeviceStreamRetention).await?,
        })
    }

    pub async fn store_pending_transactions_consumer(&self) -> Result<StorePendingTransactionsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(StorePendingTransactionsConsumer::new(self.cacher().await?))
    }

    pub async fn notifications_consumer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<NotificationsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(NotificationsConsumer::new(self.pusher(), self.stream_producer(name, shutdown_rx).await?))
    }

    pub fn notifications_failed_consumer(&self) -> NotificationsFailedConsumer {
        NotificationsFailedConsumer::new(self.database())
    }

    pub async fn in_app_notifications_consumer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<InAppNotificationsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(InAppNotificationsConsumer::new(self.database(), self.stream_producer(name, shutdown_rx).await?))
    }

    pub async fn rewards_consumer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<RewardsConsumer, Box<dyn Error + Send + Sync>> {
        Ok(RewardsConsumer::new(self.database(), self.stream_producer(name, shutdown_rx).await?))
    }

    pub async fn rewards_redemption_consumer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<RewardsRedemptionConsumer<TransferRedemptionService>, Box<dyn Error + Send + Sync>> {
        let config = self.config();
        let retry_config = RedemptionRetryConfig {
            max_retries: config.get_i64(ConfigKey::RedemptionRetryMaxRetries).await? as u32,
            delay: config.get_duration(ConfigKey::RedemptionRetryDelay).await?,
            errors: config.get_vec_string(ConfigKey::RedemptionRetryErrors).await?,
        };
        let stream_producer = self.stream_producer(name, shutdown_rx).await?;
        Ok(RewardsRedemptionConsumer::new(self.database(), Arc::new(self.redemption_service()?), retry_config, stream_producer))
    }

    pub async fn fiat_webhook_consumer(&self, name: &str, shutdown_rx: ShutdownReceiver) -> Result<FiatWebhookConsumer, Box<dyn Error + Send + Sync>> {
        let stream_producer = self.stream_producer(&format!("{name}_producer"), shutdown_rx).await?;
        let providers = self.fiat_providers(self.fiat_access_token_cacher().await?);
        Ok(FiatWebhookConsumer::new(self.database(), providers, stream_producer))
    }

    pub async fn support_webhook_consumer(&self, shutdown_rx: ShutdownReceiver) -> Result<SupportWebhookConsumer, Box<dyn Error + Send + Sync>> {
        Ok(SupportWebhookConsumer::new(self.support(shutdown_rx).await?))
    }

    async fn token_scan_providers(&self) -> Result<TokenScanProviders, Box<dyn Error + Send + Sync>> {
        let config = ScanProviderConfig::new(&self.settings().security, self.config().get_duration(ConfigKey::ScanTimeout).await?);
        let access_tokens = Arc::new(AccessTokenCacherClient::new(self.cacher().await?, GoPlusProvider::<ReqwestClient>::NAME));
        ScanProviderFactory::new_token_providers(&config, access_tokens)
    }
}
