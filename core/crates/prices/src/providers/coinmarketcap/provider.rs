use std::error::Error;

use async_trait::async_trait;
use coinmarketcap::client::CoinMarketCapClient;
use gem_client::{Client, RemoteProviderConfig, ReqwestClient, reqwest_client};
use gem_tracing::warn_with_fields;
use primitives::{Currency, FiatRate, FiatRateProvider};

use crate::FiatRatesProvider;

const CURRENCIES: &[Currency] = &[Currency::BYN, Currency::KZT, Currency::UZS, Currency::EGP, Currency::KES, Currency::COP, Currency::MAD, Currency::GHS, Currency::PEN];

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
        let mut rates = Vec::new();
        for currency in CURRENCIES {
            match self.client.get_fiat_rate(currency).await {
                Ok(rate) => rates.extend(rate),
                Err(error) => warn_with_fields!("skip failed CoinMarketCap fiat rate", currency = currency.as_ref(), error = error.as_ref()),
            }
        }
        if rates.is_empty() {
            return Err("no CoinMarketCap fiat rates".into());
        }
        Ok(rates)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::{ClientError, testkit::MockClient};

    use super::*;

    fn provider(client: MockClient) -> CoinMarketCapRatesProvider<MockClient> {
        CoinMarketCapRatesProvider {
            client: CoinMarketCapClient::new_with_client(client),
        }
    }

    fn conversion(currency: &str, price: f64) -> Vec<u8> {
        format!(r#"{{"data":{{"quote":{{"{currency}":{{"price":{price}}}}}}}}}"#).into_bytes()
    }

    #[tokio::test]
    async fn test_get_fiat_rates_keeps_currencies_that_returned() {
        let provider = provider(MockClient::new().with_get(|path| match path.rsplit_once("convert=").map(|(_, currency)| currency) {
            Some("BYN") => Ok(conversion("BYN", 3.03)),
            Some("PEN") => Ok(conversion("PEN", 3.4)),
            _ => Err(ClientError::Http { status: 429, body: b"rate limited".to_vec() }),
        }));

        assert_eq!(provider.get_fiat_rates().await.unwrap(), vec![FiatRate { symbol: Currency::BYN, rate: 3.03 }, FiatRate { symbol: Currency::PEN, rate: 3.4 }]);
    }

    #[tokio::test]
    async fn test_get_fiat_rates_fails_when_every_currency_fails() {
        let provider = provider(MockClient::new().with_get(|_| Err(ClientError::Http { status: 500, body: vec![] })));

        assert!(provider.get_fiat_rates().await.is_err());
    }
}
