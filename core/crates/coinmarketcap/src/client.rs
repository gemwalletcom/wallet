use crate::model::{Info, Listing, ListingsResponse};
use gem_client::{Client, ReqwestClient};
use serde_json::Value;
use std::{collections::HashMap, error::Error};

const COINMARKETCAP_API_URL: &str = "https://pro-api.coinmarketcap.com";
const API_KEY_HEADER: &str = "X-CMC_PRO_API_KEY";

#[derive(Debug, Clone)]
pub struct CoinMarketCapClient<C: Client> {
    client: C,
    api_key: Option<String>,
}

impl CoinMarketCapClient<ReqwestClient> {
    pub fn new_with_reqwest_client(client: reqwest::Client, api_key: &str) -> Self {
        Self::new_with_client_and_api_key(ReqwestClient::new(COINMARKETCAP_API_URL.to_string(), client), api_key)
    }
}

impl<C: Client> CoinMarketCapClient<C> {
    pub fn new_with_client(client: C) -> Self {
        Self { client, api_key: None }
    }

    pub fn new_with_client_and_api_key(client: C, api_key: &str) -> Self {
        Self {
            client,
            api_key: (!api_key.is_empty()).then_some(api_key.to_string()),
        }
    }

    pub async fn get_latest_listings(&self, limit: usize) -> Result<Vec<Listing>, Box<dyn Error + Send + Sync>> {
        self.get_listings("/v1/cryptocurrency/listings/latest", limit).await
    }

    pub async fn get_trending_latest(&self, limit: usize) -> Result<Vec<Listing>, Box<dyn Error + Send + Sync>> {
        self.get_listings("/v1/cryptocurrency/trending/latest", limit).await
    }

    pub async fn get_info_by_ids(&self, ids: &[u64]) -> Result<Vec<Info>, Box<dyn Error + Send + Sync>> {
        let ids = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
        self.get_info(&[("id".to_string(), ids)]).await
    }

    pub async fn get_info_by_id_or_symbol(&self, id_or_symbol: &str) -> Result<Vec<Info>, Box<dyn Error + Send + Sync>> {
        let key = if id_or_symbol.chars().all(|c| c.is_ascii_digit()) { "id" } else { "symbol" };
        self.get_info(&[(key.to_string(), id_or_symbol.to_string())]).await
    }

    async fn get_listings(&self, path: &str, limit: usize) -> Result<Vec<Listing>, Box<dyn Error + Send + Sync>> {
        let response: ListingsResponse = self.client.get_with(path, &[("limit".to_string(), limit.to_string())], self.headers()).await?;
        Ok(response.data)
    }

    async fn get_info(&self, query: &[(String, String)]) -> Result<Vec<Info>, Box<dyn Error + Send + Sync>> {
        let response: Value = self.client.get_with("/v2/cryptocurrency/info", query, self.headers()).await?;
        let Some(data) = response.get("data").and_then(Value::as_object) else {
            return Ok(vec![]);
        };

        let mut infos = Vec::new();
        for value in data.values() {
            if let Ok(info) = serde_json::from_value::<Info>(value.clone()) {
                infos.push(info);
            } else if let Ok(items) = serde_json::from_value::<Vec<Info>>(value.clone()) {
                infos.extend(items);
            }
        }
        Ok(infos)
    }

    fn headers(&self) -> HashMap<String, String> {
        self.api_key
            .as_ref()
            .map(|api_key| HashMap::from([(API_KEY_HEADER.to_string(), api_key.clone())]))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_get_listings_paths() {
        let client = MockClient::new().with_get(|path| {
            let body = match path {
                "/v1/cryptocurrency/listings/latest" | "/v1/cryptocurrency/trending/latest" => include_str!("../testdata/listings_latest.json"),
                _ => {
                    return Err(gem_client::ClientError::Http {
                        status: 404,
                        body: path.as_bytes().to_vec(),
                    });
                }
            };
            Ok(body.as_bytes().to_vec())
        });
        let client = CoinMarketCapClient::new_with_client(client);

        let latest = client.get_latest_listings(2).await.unwrap();
        let trending = client.get_trending_latest(2).await.unwrap();

        assert_eq!(latest.iter().map(|listing| listing.id).collect::<Vec<_>>(), vec![1027, 825]);
        assert_eq!(trending.iter().map(|listing| listing.id).collect::<Vec<_>>(), vec![1027, 825]);
    }

    #[tokio::test]
    async fn test_get_info_parses_id_and_symbol_shapes() {
        let client = MockClient::new().with_get(|path| {
            let body = match path {
                "/v2/cryptocurrency/info" => include_str!("../testdata/cryptocurrency_info.json"),
                _ => {
                    return Err(gem_client::ClientError::Http {
                        status: 404,
                        body: path.as_bytes().to_vec(),
                    });
                }
            };
            Ok(body.as_bytes().to_vec())
        });
        let client = CoinMarketCapClient::new_with_client(client);

        let infos = client.get_info_by_id_or_symbol("ETH").await.unwrap();

        assert_eq!(infos.len(), 2);
        assert!(infos.iter().any(|info| info.logo.ends_with("/1027.png")));
        assert!(infos.iter().any(|info| info.logo.ends_with("/825.png")));
    }
}
