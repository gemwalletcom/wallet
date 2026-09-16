use std::{error::Error, sync::Arc};

use pricer::PriceClient;
use prices::FiatRatesProvider;

pub struct FiatRatesUpdater {
    provider: Arc<dyn FiatRatesProvider>,
    price_client: PriceClient,
}

impl FiatRatesUpdater {
    pub fn new(provider: Arc<dyn FiatRatesProvider>, price_client: PriceClient) -> Self {
        Self { provider, price_client }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let rates = self.provider.get_fiat_rates().await?;
        self.price_client.set_fiat_rates(provider, rates).await
    }
}
