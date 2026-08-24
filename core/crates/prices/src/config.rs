use std::collections::HashMap;
use std::sync::Arc;

use gem_client::{RemoteProviderConfig, ReqwestClient};
use primitives::PriceProvider;

use crate::{CoinGeckoPricesProvider, DefiLlamaProvider, JupiterProvider, PriceAssetsProvider, PythProvider, TonApiProvider};

pub struct PriceProviderConfig {
    pub coingecko: RemoteProviderConfig,
    pub pyth: RemoteProviderConfig,
    pub jupiter: RemoteProviderConfig,
    pub defillama: RemoteProviderConfig,
    pub tonapi: RemoteProviderConfig,
    pub stonfi: RemoteProviderConfig,
}

pub type PriceProviders = HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>;

pub fn build_price_providers(config: &PriceProviderConfig, providers: impl IntoIterator<Item = PriceProvider>) -> PriceProviders {
    let client = ReqwestClient::new(String::new(), gem_client::reqwest_client());
    providers
        .into_iter()
        .map(|provider| {
            let price_provider: Arc<dyn PriceAssetsProvider> = match provider {
                PriceProvider::Coingecko => Arc::new(CoinGeckoPricesProvider::new(config.coingecko.clone())),
                PriceProvider::Pyth => Arc::new(PythProvider::new(config.pyth.configure_client(client.clone()))),
                PriceProvider::Jupiter => Arc::new(JupiterProvider::new(config.jupiter.configure_client(client.clone()))),
                PriceProvider::DefiLlama => Arc::new(DefiLlamaProvider::new(config.defillama.configure_client(client.clone()))),
                PriceProvider::TonApi => Arc::new(TonApiProvider::new(
                    config.tonapi.configure_client(client.clone()),
                    config.stonfi.configure_client(client.clone()),
                    &config.tonapi.key,
                )),
            };
            (provider, price_provider)
        })
        .collect()
}
