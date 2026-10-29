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

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let Some(name) = ens_name(query) else {
            return Ok(None);
        };
        let address = self.client.get_address(&name, chain).await?;
        Ok(address.filter(|address| !address.is_zero()).map(|address| address.to_string()))
    }
}

fn ens_name(query: &NameQuery) -> Option<String> {
    query.ascii_domain().ok().filter(|name| !name.split('.').any(|label| label.starts_with("xn--")) && try_dns_encode(name).is_ok())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, Bytes, hex};
    use alloy_sol_types::SolValue;
    use gem_client::testkit::MockClient;

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
    async fn test_resolve_network() {
        let polygon = include_str!("../../../testdata/ens_resolve_coin_type_response.json");
        let base = include_str!("../../../testdata/ens_resolve_coin_type_no_address_response.json");
        for (chain, response, expected) in [
            (Chain::Polygon, polygon, Ok(Some("0xD8EB1Fd839BD04f1A1e04b173101f7AD59Dd88EA"))),
            (Chain::Base, base, Ok(None)),
            (Chain::Base, &record_response(&[1; 32]), Err("ENS address record is not an EVM address")),
        ] {
            let response = response.to_string();
            let provider = EnsProvider::new(MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec())));

            let result = provider.resolve(&NameQuery::new("gemcoder.eth"), chain).await.map_err(|error| error.to_string());

            assert_eq!(result, expected.map(|address| address.map(str::to_string)).map_err(str::to_string), "{chain}");
        }
    }

    fn record_response(record: &[u8]) -> String {
        let result = (Bytes::from(record.to_vec()).abi_encode(), Address::ZERO).abi_encode_params();
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": hex::encode_prefixed(result)}).to_string()
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
