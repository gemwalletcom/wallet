use std::error::Error;

use async_trait::async_trait;
use gem_aptos::{AccountAddress, AptosClient, MoveOption, ViewRequest};
use gem_client::Client;
use primitives::{Chain, NameProvider};
use serde_json::json;

use crate::model::NameQuery;
use crate::resolver::NameResolver;

const ANS_ROUTER_ADDRESS: &str = "0x867ed1f6bf916171b1de3ee92849b8978b7d1b9e0a8cc982a3d19d535dfd9c0c";

pub struct AptosProvider<C: Client> {
    client: AptosClient<C>,
}

impl<C: Client> AptosProvider<C> {
    pub fn new(client: C) -> Self {
        Self { client: AptosClient::new(client) }
    }
}

#[async_trait]
impl<C: Client> NameResolver for AptosProvider<C> {
    fn provider(&self) -> NameProvider {
        NameProvider::Aptos
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["apt"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Aptos]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let Some(name) = query.domain.strip_suffix(".apt") else {
            return Ok(None);
        };
        let (subdomain, domain) = name.rsplit_once('.').map_or((None, name), |(subdomain, domain)| (Some(subdomain), domain));
        let request = ViewRequest::new(format!("{ANS_ROUTER_ADDRESS}::router::get_target_addr"), vec![json!(domain), json!(MoveOption { vec: subdomain.into_iter().collect() })]);
        let response: Vec<MoveOption<String>> = self.client.view(request).await?;
        let Some(address) = response.into_iter().next().and_then(|option| option.vec.into_iter().next()) else {
            return Ok(None);
        };
        if primitives::decode_hex(&address)?.is_empty() {
            return Err(format!("{} resolved to an empty Aptos address", query.domain).into());
        }
        Ok(Some(AccountAddress::from_hex(&address)?.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn test_resolve_pads_addresses_the_node_strips() {
        let stripped = "9e7a38edc6155ba35aecc6409c48b8fc5eb851c949594d9cdb8d0a6f9aa603e";
        let provider = AptosProvider::new(MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/v1/view");
            let request: Value = serde_json::from_slice(body).unwrap();
            assert_eq!(request["function"], format!("{ANS_ROUTER_ADDRESS}::router::get_target_addr"));
            assert_eq!(request["arguments"], serde_json::json!(["satoshi", { "vec": [] }]));
            Ok(format!(r#"[{{"vec":["0x{stripped}"]}}]"#).into_bytes())
        }));

        let resolved = provider.resolve(&NameQuery::new("satoshi.apt"), Chain::Aptos).await.unwrap().unwrap();

        assert_eq!(resolved, format!("0x0{stripped}"));
    }

    #[tokio::test]
    async fn test_resolve_keeps_full_addresses() {
        let full = "0xe687249f39dbd7e4285e74e24e786383ce851c43a3642e715fb53854bc4577b0";
        let provider = AptosProvider::new(MockClient::new().with_post(move |_, _| Ok(format!(r#"[{{"vec":["{full}"]}}]"#).into_bytes())));

        assert_eq!(provider.resolve(&NameQuery::new("gem.apt"), Chain::Aptos).await.unwrap().unwrap(), full);
    }

    #[tokio::test]
    async fn test_resolve_unnamed_has_no_record() {
        let provider = AptosProvider::new(MockClient::new().with_post(|_, _| Ok(br#"[{"vec":[]}]"#.to_vec())));

        assert_eq!(provider.resolve(&NameQuery::new("nevernevernotregistered.apt"), Chain::Aptos).await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_resolve_rejects_an_empty_address() {
        for value in ["", "0x"] {
            let provider = AptosProvider::new(MockClient::new().with_post(move |_, _| Ok(format!(r#"[{{"vec":["{value}"]}}]"#).into_bytes())));

            assert!(provider.resolve(&NameQuery::new("satoshi.apt"), Chain::Aptos).await.is_err(), "{value:?}");
        }
    }
}
