use std::collections::HashMap;
use std::sync::Arc;

use gem_client::RemoteProviderConfig;
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_reqwest::ReqwestClient;
use primitives::{FiatRateProvider, PriceProvider};

use crate::{CoinGeckoPricesProvider, CoinMarketCapRatesProvider, DefiLlamaProvider, FiatRatesProvider, HypercoreProvider, JupiterProvider, PriceAssetsProvider, PythProvider, TonApiProvider};

pub struct FiatRatesProviderConfig {
    pub coingecko: RemoteProviderConfig,
    pub coinmarketcap: RemoteProviderConfig,
}

pub fn build_fiat_rates_providers(config: &FiatRatesProviderConfig) -> HashMap<FiatRateProvider, Arc<dyn FiatRatesProvider>> {
    let providers: Vec<Arc<dyn FiatRatesProvider>> = vec![Arc::new(CoinGeckoPricesProvider::new(config.coingecko.clone())), Arc::new(CoinMarketCapRatesProvider::new(config.coinmarketcap.clone()))];
    providers.into_iter().map(|provider| (provider.provider(), provider)).collect()
}

pub struct PriceProviderConfig {
    pub coingecko: RemoteProviderConfig,
    pub pyth: RemoteProviderConfig,
    pub jupiter: RemoteProviderConfig,
    pub defillama: RemoteProviderConfig,
    pub tonapi: RemoteProviderConfig,
    pub stonfi: RemoteProviderConfig,
    pub hypercore: RemoteProviderConfig,
}

pub type PriceProviders = HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>;

pub fn build_price_providers(config: &PriceProviderConfig, providers: impl IntoIterator<Item = PriceProvider>) -> PriceProviders {
    let client = ReqwestClient::new(String::new(), gem_reqwest::reqwest_client());
    providers
        .into_iter()
        .map(|provider| {
            let price_provider: Arc<dyn PriceAssetsProvider> = match provider {
                PriceProvider::Coingecko => Arc::new(CoinGeckoPricesProvider::new(config.coingecko.clone())),
                PriceProvider::Pyth => Arc::new(PythProvider::new(client.clone().with_base_url(config.pyth.url.clone()))),
                PriceProvider::Jupiter => Arc::new(JupiterProvider::new(client.clone().with_base_url(config.jupiter.url.clone()))),
                PriceProvider::DefiLlama => Arc::new(DefiLlamaProvider::new(client.clone().with_base_url(config.defillama.url.clone()))),
                PriceProvider::TonApi => Arc::new(TonApiProvider::new(
                    client.clone().with_base_url(config.tonapi.url.clone()),
                    client.clone().with_base_url(config.stonfi.url.clone()),
                    &config.tonapi.key,
                )),
                PriceProvider::Hypercore => Arc::new(HypercoreProvider::new(HyperCoreClient::new(client.clone().with_base_url(config.hypercore.url.clone())))),
            };
            (provider, price_provider)
        })
        .collect()
}
