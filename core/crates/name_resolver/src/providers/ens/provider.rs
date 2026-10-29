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
    pub fn new(client: C, ccip: C, ccip_url: String) -> Self {
        Self {
            client: EnsClient::new(client, ccip, ccip_url),
        }
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
    use alloy_sol_types::{SolError, SolValue};
    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use serde_json::Value;

    use super::super::testkit::{CCIP_URL, offchain_lookup};
    use super::*;

    const LOOKUP_RESPONSE: &[u8] = include_bytes!("../../../testdata/ens_offchain_lookup_response.json");
    const CCIP_RESPONSE: &[u8] = include_bytes!("../../../testdata/ens_ccip_response.json");

    fn ens_provider(client: MockClient, ccip: MockClient) -> EnsProvider<MockClient> {
        EnsProvider::new(client, ccip, CCIP_URL.to_string())
    }

    fn offchain_rpc(callback_response: Vec<u8>) -> MockClient {
        let lookup = offchain_lookup();
        let ccip_data = hex::decode(serde_json::from_slice::<Value>(CCIP_RESPONSE).unwrap()["data"].as_str().unwrap()).unwrap();
        let callback = hex::encode_prefixed([lookup.callbackFunction.as_slice(), &(Bytes::from(ccip_data), lookup.extraData).abi_encode_params()].concat());
        MockClient::new().with_post(move |_, body| {
            let data = serde_json::from_slice::<Value>(body).unwrap()["params"][0]["data"].take();
            Ok(if data == callback.as_str() { callback_response.clone() } else { LOOKUP_RESPONSE.to_vec() })
        })
    }

    #[tokio::test]
    async fn test_resolve_offchain() {
        let ccip = MockClient::new().with_post(|_, _| Ok(CCIP_RESPONSE.to_vec()));
        let ccip_error = MockClient::new().with_post(|_, _| Err(ClientError::Http { status: 500, body: vec![] }));
        for (callback_response, ccip, expected) in [
            (include_bytes!("../../../testdata/ens_offchain_resolve_response.json").to_vec(), ccip.clone(), Ok("0x179A862703a4adfb29896552DF9e307980D19285")),
            (include_bytes!("../../../testdata/ens_resolver_error_response.json").to_vec(), ccip.clone(), Err("execution reverted (3)")),
            (LOOKUP_RESPONSE.to_vec(), ccip, Err("ENS offchain lookup limit exceeded")),
            (vec![], ccip_error, Err("HTTP error: status 500")),
        ] {
            let provider = ens_provider(offchain_rpc(callback_response), ccip);

            let result = provider.resolve(&NameQuery::new("gregskril.uni.eth"), Chain::Ethereum).await.map_err(|error| error.to_string());

            assert_eq!(result, expected.map(|address| Some(address.to_string())).map_err(str::to_string));
        }
    }

    #[tokio::test]
    async fn test_resolve_offchain_sender_must_match() {
        let mut lookup = offchain_lookup();
        lookup.sender = Address::ZERO;
        let response = serde_json::json!({"jsonrpc": "2.0", "id": 1, "error": {"code": 3, "message": "execution reverted", "data": hex::encode_prefixed(lookup.abi_encode())}}).to_string();
        let provider = ens_provider(MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec())), MockClient::new());

        let error = provider.resolve(&NameQuery::new("gregskril.uni.eth"), Chain::Ethereum).await.unwrap_err();

        assert_eq!(error.to_string(), "ENS offchain lookup sender does not match Universal Resolver");
    }

    #[tokio::test]
    async fn test_resolve() {
        for (name, response, expected) in [
            ("vitalik.eth", include_str!("../../../testdata/ens_resolve_response.json"), Ok(Some("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))),
            ("dao.eth", include_str!("../../../testdata/ens_resolve_no_address_response.json"), Ok(None)),
            ("nevernevernotregistered12345.eth", include_str!("../../../testdata/ens_resolver_not_found_response.json"), Ok(None)),
            ("registrar.eth", include_str!("../../../testdata/ens_resolver_error_response.json"), Err("execution reverted (3)")),
            ("vitalik.eth", r#"{"jsonrpc":"2.0","id":1,"error":{"code":3,"message":"execution reverted"}}"#, Err("execution reverted (3)")),
        ] {
            let provider = ens_provider(MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec())), MockClient::new());

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
            let provider = ens_provider(MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec())), MockClient::new());

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
