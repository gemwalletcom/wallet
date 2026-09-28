use prices::{PriceProvider, PriceProviderConfig, PriceProviders, build_price_providers};

use crate::Services;

impl Services {
    pub fn price_providers(&self, providers: impl IntoIterator<Item = PriceProvider>) -> PriceProviders {
        let prices = &self.settings().prices;
        build_price_providers(
            &PriceProviderConfig {
                coingecko: prices.coingecko.remote_provider_config(),
                pyth: prices.pyth.remote_provider_config(),
                jupiter: prices.jupiter.remote_provider_config(),
                defillama: prices.defillama.remote_provider_config(),
                tonapi: prices.tonapi.remote_provider_config(),
                stonfi: prices.stonfi.remote_provider_config(),
            },
            providers,
        )
    }
}
