use gem_client::Client;
use primitives::{ApplicationMetadata, ApplicationMetadataSource, Chain, ChainAddress};

use crate::provider::PaymentProvider;
use crate::solana_pay::client::SolanaPayClient;
use crate::{PaymentError, PaymentTransaction};

#[derive(Debug)]
pub(crate) struct SolanaPayProvider<C: Client> {
    client: SolanaPayClient<C>,
    url: String,
}

impl<C: Client> SolanaPayProvider<C> {
    pub(crate) fn new(client: C, url: String) -> Self {
        Self {
            client: SolanaPayClient::new(client),
            url,
        }
    }
}

impl<C: Client> PaymentProvider for SolanaPayProvider<C> {
    fn supported_chains(&self) -> &'static [Chain] {
        &[Chain::Solana]
    }

    async fn load(&self, addresses: &[ChainAddress]) -> Result<PaymentTransaction, PaymentError> {
        let account = addresses
            .iter()
            .find(|address| address.chain == Chain::Solana)
            .cloned()
            .ok_or(PaymentError::NoPaymentOptions)?;
        let (info, response) = futures::try_join!(self.client.get_info(), self.client.get_transaction(&account.address))?;
        let prepared = crate::solana_pay::transaction::prepare(&response.transaction, &account.address).map_err(|reason| PaymentError::InvalidRequest { reason })?;

        Ok(PaymentTransaction {
            merchant: ApplicationMetadata {
                name: info.label,
                description: String::new(),
                url: self.url.clone(),
                icon: info.icon,
                source: ApplicationMetadataSource::Payment,
            },
            account,
            transaction: prepared.transaction,
            transaction_type: prepared.transaction_type,
            memo: prepared.memo,
            request: prepared.request,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    const ACCOUNT: &str = "mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN";
    const TRANSACTION: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAECC4JMKqNplIXybGb/GhK1ofdVWeuEjXnQor7gi0Y2hMcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQECAAAMAgAAAAAAAAAAAAAA";

    fn addresses() -> Vec<ChainAddress> {
        vec![ChainAddress::new(Chain::Solana, ACCOUNT.to_string())]
    }

    fn provider() -> SolanaPayProvider<MockClient> {
        SolanaPayProvider::new(
            MockClient::new()
                .with_get(|path| {
                    assert_eq!(path, "");
                    Ok(br#"{"label":"Constant K","icon":"https://constant-k.com/icon.png"}"#.to_vec())
                })
                .with_post(|path, body| {
                    assert_eq!(path, "");
                    assert_eq!(serde_json::from_slice::<serde_json::Value>(body).unwrap(), serde_json::json!({ "account": ACCOUNT }));
                    Ok(format!(r#"{{"message":"Annual plan","transaction":"{TRANSACTION}"}}"#).into_bytes())
                }),
            "https://constant-k.com/checkout".to_string(),
        )
    }

    #[tokio::test]
    async fn test_transaction_request() {
        let transaction = provider().load(&addresses()).await.unwrap();

        assert_eq!(transaction.merchant.name, "Constant K");
        assert!(!transaction.transaction.is_empty());
    }
}
