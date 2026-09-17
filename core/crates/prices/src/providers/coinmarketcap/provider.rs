use std::error::Error;

use async_trait::async_trait;
use coinmarketcap::client::CoinMarketCapClient;
use gem_client::{Client, RemoteProviderConfig, ReqwestClient, reqwest_client};
use primitives::{Currency, FiatRate, FiatRateProvider};

use crate::FiatRatesProvider;

const CURRENCIES: &[Currency] = &[
    Currency::BYN,
    Currency::KZT,
    Currency::UZS,
    Currency::EGP,
    Currency::KES,
    Currency::COP,
    Currency::MAD,
    Currency::GHS,
    Currency::PEN,
];

pub struct CoinMarketCapRatesProvider<C: Client = ReqwestClient> {
    client: CoinMarketCapClient<C>,
}

impl CoinMarketCapRatesProvider<ReqwestClient> {
    pub fn new(config: RemoteProviderConfig) -> Self {
        Self {
            client: CoinMarketCapClient::new_with_reqwest_client(reqwest_client(), config),
        }
    }
}

#[async_trait]
impl<C: Client + 'static> FiatRatesProvider for CoinMarketCapRatesProvider<C> {
    fn provider(&self) -> FiatRateProvider {
        FiatRateProvider::Coinmarketcap
    }

    async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        self.client.get_fiat_rates(CURRENCIES).await
    }
}
