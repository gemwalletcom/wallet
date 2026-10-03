use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient, RateLimiter};
use config_keys::{ConfigKey, RateLimitKey};
use fiat::error::FiatQuoteError;
use fiat::model::{FiatMapping, FiatMappingMap};
use fiat::quotes::{compare_quotes, get_provider_quote, is_country_allowed, is_provider_eligible};
use fiat::{FiatDeviceContext, FiatProvider, FiatWebhookRequest, IPAddressInfo, IPCheckClient};
use futures::future::join_all;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{
    Asset, AssetId, Chain, FiatAsset, FiatAssetSymbol, FiatAssets, FiatQuoteError as ProviderQuoteError, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData, FiatQuotes, FiatTransaction, FiatTransactionData, FiatWebhook,
    RequestError,
};
use storage::{AssetFilter, AssetsRepository, Database, DatabaseError, DevicesRepository, FiatRepository, WalletAddress, WalletsRepository};
use streamer::{FiatWebhookPayload, QueueName, StreamProducer};

use super::fiat_cacher_client::{CachedFiatQuote, FiatCacherClient};
use crate::ConfigCacher;

pub struct FiatClient {
    database: Database,
    config: Arc<ConfigCacher>,
    cacher: CacherClient,
    fiat_cacher: FiatCacherClient,
    rate_limiter: RateLimiter,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    ip_check_client: IPCheckClient,
    stream_producer: StreamProducer,
}

impl FiatClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>, cacher: CacherClient, providers: Vec<Box<dyn FiatProvider + Send + Sync>>, ip_check_client: IPCheckClient, stream_producer: StreamProducer) -> Self {
        Self {
            database,
            config,
            fiat_cacher: FiatCacherClient::new(cacher.clone()),
            rate_limiter: RateLimiter::new(cacher.clone()),
            cacher,
            providers,
            ip_check_client,
            stream_producer,
        }
    }

    pub async fn get_on_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        self.get_assets(AssetFilter::IsBuyable(true)).await
    }

    pub async fn get_off_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        self.get_assets(AssetFilter::IsSellable(true)).await
    }

    pub async fn get_quote_assets(&self, quote_type: FiatQuoteType) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        match quote_type {
            FiatQuoteType::Buy => self.get_on_ramp_assets().await,
            FiatQuoteType::Sell => self.get_off_ramp_assets().await,
        }
    }

    pub async fn get_transactions_by_device_wallet_id(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let transactions = self.database.run(move |client| client.get_fiat_transactions_by_device_and_wallet_id(device_row_id, wallet_id)).await?;
        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }

    pub async fn get_transactions_by_device_id(&self, device_id: &str) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let transactions = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let device_row_id = client.get_device_row_id(&device_id)?;
                client.get_fiat_transactions_by_device_id(device_row_id)
            })
            .await?;
        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }

    pub async fn publish_webhook(&self, request: FiatWebhookRequest, provider_name: &str) -> Result<FiatWebhookPayload, Box<dyn Error + Send + Sync>> {
        let provider = self.provider(provider_name)?;
        let name = provider.name();
        let provider_id = name.id();
        let webhook_data = request.data.clone();
        let webhook = provider.parse_webhook(request).await.map_err(|error| {
            if matches!(error.downcast_ref(), Some(FiatQuoteError::InvalidWebhook)) {
                error_with_fields!("invalid fiat webhook payload", &*error, provider = provider_id, payload = format!("{webhook_data:#}"));
            } else {
                error_with_fields!("rejected fiat webhook", &*error, provider = provider_id);
            }
            error
        })?;

        let (kind, transaction_id) = match &webhook {
            FiatWebhook::OrderId(order_id) => ("order_id", Some(order_id.clone())),
            FiatWebhook::Transaction(transaction) => ("transaction", transaction.provider_transaction_id.clone().or_else(|| Some(transaction.transaction_id.clone()))),
            FiatWebhook::None => ("none", None),
        };
        let transaction_id = transaction_id.unwrap_or_default();

        info_with_fields!("received fiat webhook", provider = provider_id, kind = kind, transaction_id = transaction_id.as_str());

        let payload = FiatWebhookPayload::new(name, webhook_data, webhook);
        match payload.payload {
            FiatWebhook::OrderId(_) | FiatWebhook::Transaction(_) => {
                self.stream_producer.publish(QueueName::FiatOrderWebhooks, &payload).await?;
                info_with_fields!("published fiat webhook", provider = provider_id, transaction_id = transaction_id.as_str());
            }
            FiatWebhook::None => {
                info_with_fields!("ignored fiat webhook", provider = provider_id);
            }
        }
        Ok(payload)
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        let asset = self.get_asset(&request.asset_id).await?;
        let (quotes, errors) = self.get_provider_quotes(&request, &asset, &request.ip_address).await?;
        Ok(FiatQuotes {
            quotes: quotes.into_iter().map(|quote| quote.quote).collect(),
            errors,
        })
    }

    pub async fn get_device_quotes(&self, request: FiatQuoteRequest, context: &FiatDeviceContext) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        context.validate_wallet()?;
        self.consume_limits(context, RateLimitKey::FiatQuoteRequestPerDeviceLimit, RateLimitKey::FiatQuoteRequestPerIpLimit).await?;
        let asset = self.get_asset(&request.asset_id).await?;
        if self.config.get_bool(ConfigKey::FiatValidateSubscription).await? {
            self.subscription_address(context, asset.chain()).await?;
        }
        let (quotes, errors) = self.get_provider_quotes(&request, &asset, &context.ip_address).await?;
        Ok(FiatQuotes {
            quotes: self.fiat_cacher.set_quotes(context, quotes).await?,
            errors,
        })
    }

    pub async fn get_quote_url(&self, quote_id: &str, context: &FiatDeviceContext, locale: &str) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        context.validate_wallet()?;
        self.consume_limits(context, RateLimitKey::FiatQuoteUrlRequestPerDeviceLimit, RateLimitKey::FiatQuoteUrlRequestPerIpLimit).await?;
        let cached_quote = self.fiat_cacher.get_quote(context, quote_id).await?;
        if let Some(url) = cached_quote.url.clone() {
            return Ok(url);
        }
        self.create_quote_url(quote_id, context, locale, cached_quote).await
    }

    fn provider(&self, provider_name: &str) -> Result<&(dyn FiatProvider + Send + Sync), Box<dyn Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|provider| provider.name().id() == provider_name)
            .map(AsRef::as_ref)
            .ok_or_else(|| format!("Provider {} not found", provider_name).into())
    }

    async fn get_assets(&self, filter: AssetFilter) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        let assets = self.database.run(move |client| client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), filter])).await?;
        Ok(FiatAssets::new(assets.into_iter().map(|asset| asset.asset.id.to_string()).collect()))
    }

    async fn get_provider_quotes(&self, request: &FiatQuoteRequest, asset: &Asset, ip_address: &str) -> Result<(Vec<CachedFiatQuote>, Vec<ProviderQuoteError>), Box<dyn Error + Send + Sync>> {
        let asset_id = asset.id.clone();
        let (providers_countries, fiat_assets, db_providers) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> { Ok((client.get_fiat_providers_countries()?, client.get_fiat_assets_for_asset_id(&asset_id)?, client.get_fiat_providers()?)) })
            .await?;
        let ip_address_info = self.get_ip_address(ip_address).await.map_err(|error| format!("IP address validation failed: {error}"))?;
        let fiat_mapping_map = fiat_mapping(asset, request.quote_type, fiat_assets);
        let country_code = &ip_address_info.alpha2;
        if !is_country_allowed(&providers_countries, country_code, request.provider_id.as_deref()) {
            return Err(FiatQuoteError::RegionUnavailable.into());
        }

        let requests = self.providers.iter().filter(|provider| request.provider_id.as_deref().is_none_or(|id| provider.name().id() == id)).filter_map(|provider| {
            let provider_name = provider.name();
            let db_provider = db_providers.iter().find(|db_provider| db_provider.id == provider_name)?;
            let countries: HashSet<String> = providers_countries
                .iter()
                .filter(|country| country.provider == provider_name && country.is_allowed)
                .map(|country| country.alpha2.clone())
                .collect();
            let mapping = fiat_mapping_map.get(provider_name.id());
            if !is_provider_eligible(db_provider, &countries, mapping, country_code, request) {
                return None;
            }
            let mapping = mapping?;
            Some(async move {
                get_provider_quote(provider.as_ref(), request, asset, mapping, &db_provider.payment_methods)
                    .await
                    .map(|quote| CachedFiatQuote {
                        quote,
                        asset_symbol: mapping.asset_symbol.clone(),
                        country_code: Some(country_code.clone()),
                        url: None,
                    })
                    .map_err(|error| ProviderQuoteError::new(Some(provider_name.id().to_string()), error.to_string()))
            })
        });

        let mut quotes = Vec::new();
        let mut errors = Vec::new();
        for result in join_all(requests).await {
            match result {
                Ok(quote) => quotes.push(quote),
                Err(error) => errors.push(error),
            }
        }
        quotes.sort_by(|a, b| compare_quotes(request.quote_type, &a.quote, &b.quote, &db_providers));

        Ok((quotes, errors))
    }

    async fn create_quote_url(&self, quote_id: &str, context: &FiatDeviceContext, locale: &str, cached_quote: CachedFiatQuote) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let CachedFiatQuote { quote, asset_symbol, country_code, .. } = cached_quote;
        let provider = self.provider(quote.provider.id.as_ref())?;
        let wallet_address = self.subscription_address(context, quote.asset.chain()).await?;
        let data = FiatQuoteUrlData {
            quote,
            asset_symbol,
            wallet_address: wallet_address.address,
            ip_address: context.ip_address.clone(),
            locale: locale.to_string(),
        };

        let url = provider.get_quote_url(data.clone()).await?;
        let country = match country_code {
            Some(country_code) => country_code,
            None => self.get_ip_address(&context.ip_address).await?.alpha2,
        };
        let pending_transaction = FiatTransaction::new_pending(&data, Some(country), url.provider_transaction_id.clone());
        let (device_id, wallet_id, address_id) = (context.device_id, context.wallet_id, wallet_address.id);
        self.database.run(move |client| client.add_fiat_transaction(pending_transaction, device_id, wallet_id, address_id)).await?;
        self.fiat_cacher.set_quote_url(context, quote_id, &url).await?;

        Ok(url)
    }

    async fn get_asset(&self, asset_id: &AssetId) -> Result<Asset, DatabaseError> {
        let asset_id = asset_id.clone();
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn subscription_address(&self, context: &FiatDeviceContext, chain: Chain) -> Result<WalletAddress, Box<dyn Error + Send + Sync>> {
        let (device_id, wallet_id) = (context.device_id, context.wallet_id);
        match self.database.run(move |client| client.subscriptions_wallet_address_for_chain(device_id, wallet_id, chain)).await {
            Ok(address) => Ok(address),
            Err(error) if error.is_not_found() => Err(RequestError::Forbidden.into()),
            Err(error) => Err(error.into()),
        }
    }

    async fn consume_limits(&self, context: &FiatDeviceContext, device_key: RateLimitKey, ip_key: RateLimitKey) -> Result<(), Box<dyn Error + Send + Sync>> {
        let device_id = context.device_id.to_string();
        let mut allowed = true;
        for (key, scope) in [(device_key, device_id.as_str()), (ip_key, context.ip_address.as_str())] {
            allowed &= self.rate_limiter.consume(key, scope, self.config.get_rate_limit(key).await?).await?;
        }
        if !allowed {
            return Err(RequestError::LimitReached.into());
        }
        Ok(())
    }

    async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        self.cacher.get_or_set_cached(CacheKey::FiatIpCheck(ip_address), || self.ip_check_client.get_ip_address(ip_address)).await
    }
}

fn fiat_mapping(asset: &Asset, quote_type: FiatQuoteType, fiat_assets: Vec<FiatAsset>) -> FiatMappingMap {
    fiat_assets
        .into_iter()
        .filter(|fiat_asset| match quote_type {
            FiatQuoteType::Buy => fiat_asset.is_buy_enabled,
            FiatQuoteType::Sell => fiat_asset.is_sell_enabled,
        })
        .map(|fiat_asset| {
            (
                fiat_asset.provider.id().to_string(),
                FiatMapping {
                    asset: asset.clone(),
                    asset_symbol: FiatAssetSymbol {
                        symbol: fiat_asset.symbol,
                        network: fiat_asset.network,
                    },
                    unsupported_countries: fiat_asset.unsupported_countries,
                    buy_limits: fiat_asset.buy_limits,
                    sell_limits: fiat_asset.sell_limits,
                },
            )
        })
        .collect()
}
