use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{CachedFiatQuote, FiatQuoteCacher, RateLimitCacher};
use config_keys::{ConfigKey, RateLimitKey};
use fiat::error::FiatQuoteError;
use fiat::model::{FiatMapping, FiatMappingMap};
use fiat::quotes::{compare_quotes, get_provider_quote, is_country_allowed, is_provider_eligible};
use fiat::{FiatDeviceContext, FiatProvider, FiatWebhookRequest, IpAddressProvider};
use futures::future::join_all;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{
    Asset, AssetId, Chain, FiatAsset, FiatAssetSymbol, FiatAssets, FiatQuote, FiatQuoteError as ProviderQuoteError, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData, FiatQuotes, FiatTransaction, FiatTransactionData,
    FiatWebhook, RequestError,
};
use storage::{DatabaseError, WalletAddress};
use streamer::{FiatWebhookPayload, StreamProducerQueue};
use uuid::Uuid;

use super::error::FiatServiceError;
use super::repository::{QuoteContext, Repository};
use crate::ConfigCacher;
use crate::assets::AssetCatalogClient;

pub struct FiatClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
    quote_cacher: Arc<dyn FiatQuoteCacher>,
    rate_limiter: Arc<dyn RateLimitCacher>,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    ip_address_provider: Arc<dyn IpAddressProvider>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    asset_catalog: Arc<AssetCatalogClient>,
}

impl FiatClient {
    pub(crate) fn new(
        repository: Arc<dyn Repository>,
        config: Arc<ConfigCacher>,
        quote_cacher: Arc<dyn FiatQuoteCacher>,
        rate_limiter: Arc<dyn RateLimitCacher>,
        providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
        ip_address_provider: Arc<dyn IpAddressProvider>,
        stream_producer: Arc<dyn StreamProducerQueue>,
        asset_catalog: Arc<AssetCatalogClient>,
    ) -> Self {
        Self {
            repository,
            config,
            quote_cacher,
            rate_limiter,
            providers,
            ip_address_provider,
            stream_producer,
            asset_catalog,
        }
    }

    pub async fn get_on_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        Ok(self.asset_catalog.get().await?.fiat_on_ramp_assets)
    }

    pub async fn get_off_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        Ok(self.asset_catalog.get().await?.fiat_off_ramp_assets)
    }

    pub async fn get_quote_assets(&self, quote_type: FiatQuoteType) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        match quote_type {
            FiatQuoteType::Buy => self.get_on_ramp_assets().await,
            FiatQuoteType::Sell => self.get_off_ramp_assets().await,
        }
    }

    pub async fn get_transactions_by_device_wallet_id(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let transactions = self.repository.wallet_fiat_transactions(device_row_id, wallet_id).await?;
        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }

    pub async fn get_transactions_by_device_id(&self, device_id: &str) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let transactions = self.repository.device_fiat_transactions(device_id.to_string()).await?;
        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }

    pub async fn publish_webhook(&self, request: FiatWebhookRequest, provider_name: &str) -> Result<FiatWebhookPayload, FiatServiceError> {
        let provider = self.provider(provider_name)?;
        let name = provider.name();
        let provider_id = name.id();
        let webhook_data = request.data.clone();
        let webhook = provider.parse_webhook(request).await.map_err(|error| {
            let error = FiatServiceError::provider(error);
            if matches!(error, FiatServiceError::Quote(FiatQuoteError::InvalidWebhook)) {
                error_with_fields!("invalid fiat webhook payload", &error, provider = provider_id, payload = format!("{webhook_data:#}"));
            } else {
                error_with_fields!("rejected fiat webhook", &error, provider = provider_id);
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
                self.stream_producer.publish_fiat_webhook(payload.clone()).await?;
                info_with_fields!("published fiat webhook", provider = provider_id, transaction_id = transaction_id.as_str());
            }
            FiatWebhook::None => {
                info_with_fields!("ignored fiat webhook", provider = provider_id);
            }
        }
        Ok(payload)
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, FiatServiceError> {
        let asset = self.get_asset(&request.asset_id).await?;
        let (quotes, errors) = self.get_provider_quotes(&request, &asset, &request.ip_address).await?;
        Ok(FiatQuotes {
            quotes: quotes.into_iter().map(|quote| quote.quote).collect(),
            errors,
        })
    }

    pub async fn get_device_quotes(&self, request: FiatQuoteRequest, context: &FiatDeviceContext) -> Result<FiatQuotes, FiatServiceError> {
        context.validate_wallet()?;
        self.consume_limits(context, RateLimitKey::FiatQuoteRequestPerDeviceLimit, RateLimitKey::FiatQuoteRequestPerIpLimit).await?;
        let asset = self.get_asset(&request.asset_id).await?;
        if self.config.get_bool(ConfigKey::FiatValidateSubscription).await? {
            self.subscription_address(context, asset.chain()).await?;
        }
        let (quotes, errors) = self.get_provider_quotes(&request, &asset, &context.ip_address).await?;
        Ok(FiatQuotes {
            quotes: self.add_quotes(context, quotes).await?,
            errors,
        })
    }

    pub async fn get_quote_url(&self, quote_id: &str, context: &FiatDeviceContext, locale: &str) -> Result<FiatQuoteUrl, FiatServiceError> {
        context.validate_wallet()?;
        self.consume_limits(context, RateLimitKey::FiatQuoteUrlRequestPerDeviceLimit, RateLimitKey::FiatQuoteUrlRequestPerIpLimit).await?;
        let cached_quote = self.cached_quote(context, quote_id).await?;
        if let Some(url) = cached_quote.url.clone() {
            return Ok(url);
        }
        self.create_quote_url(quote_id, context, locale, cached_quote).await
    }

    fn provider(&self, provider_name: &str) -> Result<&(dyn FiatProvider + Send + Sync), FiatServiceError> {
        self.providers
            .iter()
            .find(|provider| provider.name().id() == provider_name)
            .map(AsRef::as_ref)
            .ok_or_else(|| FiatServiceError::Internal(format!("Provider {provider_name} not found").into()))
    }

    async fn get_provider_quotes(&self, request: &FiatQuoteRequest, asset: &Asset, ip_address: &str) -> Result<(Vec<CachedFiatQuote>, Vec<ProviderQuoteError>), FiatServiceError> {
        let QuoteContext {
            countries: providers_countries,
            fiat_assets,
            providers: db_providers,
        } = self.repository.quote_context(asset.id.clone()).await?;
        let ip_address_info = self
            .ip_address_provider
            .get_ip_address(ip_address)
            .await
            .map_err(|error| FiatServiceError::Internal(format!("IP address validation failed: {error}").into()))?;
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

    async fn create_quote_url(&self, quote_id: &str, context: &FiatDeviceContext, locale: &str, cached_quote: CachedFiatQuote) -> Result<FiatQuoteUrl, FiatServiceError> {
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

        let url = provider.get_quote_url(data.clone()).await.map_err(|error| {
            info_with_fields!("fiat quote url failed", provider = data.quote.provider.id.as_ref(), quote_id = quote_id, error = error.to_string());
            FiatServiceError::provider(error)
        })?;
        let country = match country_code {
            Some(country_code) => country_code,
            None => self.ip_address_provider.get_ip_address(&context.ip_address).await?.alpha2,
        };
        let pending_transaction = FiatTransaction::new_pending(&data, Some(country), url.provider_transaction_id.clone());
        self.repository.add_fiat_transaction(pending_transaction, context.device_id, context.wallet_id, wallet_address.id).await?;
        let cached_quote = self.cached_quote(context, quote_id).await?;
        let quote = CachedFiatQuote { url: Some(url.clone()), ..cached_quote };
        self.quote_cacher.set_quotes(context.device_id, context.wallet_id, &[(quote_id.to_string(), quote)]).await?;

        Ok(url)
    }

    async fn add_quotes(&self, context: &FiatDeviceContext, quotes: Vec<CachedFiatQuote>) -> Result<Vec<FiatQuote>, FiatServiceError> {
        let scoped_quotes: Vec<_> = quotes.into_iter().map(|quote| (Uuid::new_v4().to_string(), quote)).collect();
        self.quote_cacher.set_quotes(context.device_id, context.wallet_id, &scoped_quotes).await?;
        Ok(scoped_quotes.into_iter().map(|(quote_id, cached_quote)| FiatQuote { id: quote_id, ..cached_quote.quote }).collect())
    }

    async fn cached_quote(&self, context: &FiatDeviceContext, quote_id: &str) -> Result<CachedFiatQuote, FiatServiceError> {
        match self.quote_cacher.quote(context.device_id, context.wallet_id, quote_id).await? {
            Some(quote) => Ok(quote),
            None => Err(RequestError::Forbidden.into()),
        }
    }

    async fn get_asset(&self, asset_id: &AssetId) -> Result<Asset, DatabaseError> {
        self.repository.asset(asset_id.clone()).await
    }

    async fn subscription_address(&self, context: &FiatDeviceContext, chain: Chain) -> Result<WalletAddress, FiatServiceError> {
        match self.repository.subscription_address(context.device_id, context.wallet_id, chain).await {
            Ok(address) => Ok(address),
            Err(error) if error.is_not_found() => Err(RequestError::Forbidden.into()),
            Err(error) => Err(error.into()),
        }
    }

    async fn consume_limits(&self, context: &FiatDeviceContext, device_key: RateLimitKey, ip_key: RateLimitKey) -> Result<(), FiatServiceError> {
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
