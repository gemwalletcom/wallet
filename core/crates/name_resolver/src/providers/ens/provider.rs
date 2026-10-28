use std::error::Error;

use alloy_ens::try_dns_encode;
use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, NameProvider};

use super::client::EnsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct EnsProvider<C: Client + Clone> {
    client: EnsClient<C>,
}

impl<C: Client + Clone> EnsProvider<C> {
    pub fn new(client: C) -> Self {
        Self { client: EnsClient::new(client) }
    }
}

#[async_trait]
impl<C: Client + Clone> NameResolver for EnsProvider<C> {
    fn provider(&self) -> NameProvider {
        NameProvider::Ens
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["eth", "com", "xyz", "dev"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::SmartChain,
            Chain::Polygon,
            Chain::Optimism,
            Chain::Arbitrum,
            Chain::Base,
            Chain::AvalancheC,
            Chain::Fantom,
            Chain::Gnosis,
        ]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let Some(name) = ens_name(query) else {
            return Ok(None);
        };
        let address = self.client.get_address(&name).await?;
        Ok(address.filter(|address| !address.is_zero()).map(|address| address.to_string()))
    }
}

fn ens_name(query: &NameQuery) -> Option<String> {
    query.ascii_domain().ok().filter(|name| !name.split('.').any(|label| label.starts_with("xn--")) && try_dns_encode(name).is_ok())
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn test_resolve() {
        for (name, response, expected) in [
            ("vitalik.eth", include_str!("../../../testdata/ens_resolve_response.json"), Ok(Some("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))),
            ("dao.eth", include_str!("../../../testdata/ens_resolve_no_address_response.json"), Ok(None)),
            ("nevernevernotregistered12345.eth", include_str!("../../../testdata/ens_resolver_not_found_response.json"), Ok(None)),
            ("gregskril.uni.eth", include_str!("../../../testdata/ens_offchain_lookup_response.json"), Ok(None)),
            ("registrar.eth", include_str!("../../../testdata/ens_resolver_error_response.json"), Err("execution reverted (3)")),
            ("vitalik.eth", r#"{"jsonrpc":"2.0","id":1,"error":{"code":3,"message":"execution reverted"}}"#, Err("execution reverted (3)")),
        ] {
            let provider = EnsProvider::new(MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec())));

            let result = provider.resolve(&NameQuery::new(name), Chain::Ethereum).await.map_err(|error| error.to_string());

            assert_eq!(result, expected.map(|address| address.map(str::to_string)).map_err(str::to_string), "{name}");
        }
    }

    #[tokio::test]
    async fn test_resolve_request() {
        let provider = EnsProvider::new(MockClient::new().with_post(|_, body| {
            let params = serde_json::from_slice::<Value>(body).unwrap()["params"].take();
            assert_eq!(params, serde_json::from_str::<Value>(include_str!("../../../testdata/ens_resolve_request.json")).unwrap());
            Ok(include_bytes!("../../../testdata/ens_resolve_response.json").to_vec())
        }));

        provider.resolve(&NameQuery::new("vitalik.eth"), Chain::Ethereum).await.unwrap();
    }

    #[test]
    fn test_ens_name() {
        assert_eq!(ens_name(&NameQuery::new("Vitalik.ETH")).as_deref(), Some("vitalik.eth"));
        assert_eq!(ens_name(&NameQuery::new("🐈.eth")), None);
        assert_eq!(ens_name(&NameQuery::new("xn--zn8h.eth")), None);
        assert_eq!(ens_name(&NameQuery::new("mañana.eth")), None);
        assert_eq!(ens_name(&NameQuery::new("a..eth")), None);
        assert_eq!(ens_name(&NameQuery::new(".eth")), None);
        assert_eq!(ens_name(&NameQuery::new("eth.")), None);
    }
}
