use crate::model::{PositionsResponse, Token};
use gem_client::{Client, ClientExt, ReqwestClient};
use std::{collections::HashMap, error::Error};

const JUPITER_API_URL: &str = "https://api.jup.ag";
const JUPITER_API_HEADER_KEY: &str = "x-api-key";

#[derive(Debug, Clone)]
pub struct JupiterClient<C: Client> {
    client: C,
    api_key: Option<String>,
}

impl JupiterClient<ReqwestClient> {
    pub fn new_with_reqwest_client(client: reqwest::Client) -> Self {
        Self::new_with_client(ReqwestClient::new(JUPITER_API_URL.to_string(), client))
    }

    pub fn new_with_reqwest_client_and_api_key(client: reqwest::Client, api_key: String) -> Self {
        Self::new_with_client_and_api_key(ReqwestClient::new(JUPITER_API_URL.to_string(), client), api_key)
    }
}

impl<C: Client> JupiterClient<C> {
    pub fn new_with_client(client: C) -> Self {
        Self { client, api_key: None }
    }

    pub fn new_with_client_and_api_key(client: C, api_key: String) -> Self {
        Self {
            client,
            api_key: (!api_key.is_empty()).then_some(api_key),
        }
    }

    pub async fn get_verified_tokens(&self) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_with_query("/tokens/v2/tag", &[("query".to_string(), "verified".to_string())]).await?)
    }

    pub async fn get_top_trending_tokens(&self, interval: &str, limit: usize) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        let path = format!("/tokens/v2/toptrending/{interval}");
        Ok(self.client.get_with_query(&path, &[("limit".to_string(), limit.to_string())]).await?)
    }

    pub async fn get_wallet_positions(&self, address: &str) -> Result<PositionsResponse, Box<dyn Error + Send + Sync>> {
        let path = format!("/portfolio/v1/positions/{address}");
        Ok(self.client.get_with_headers(&path, self.headers()).await?)
    }

    fn headers(&self) -> HashMap<String, String> {
        self.api_key
            .as_ref()
            .map(|api_key| HashMap::from([(JUPITER_API_HEADER_KEY.to_string(), api_key.clone())]))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_get_tokens_paths() {
        let client = MockClient::new().with_get(|path| {
            let body = match path {
                "/tokens/v2/tag" => r#"[{"id":"So11111111111111111111111111111111111111112","icon":"https://example.com/sol.png","isVerified":true}]"#,
                "/tokens/v2/toptrending/24h" => r#"[{"id":"JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN","icon":"https://example.com/jup.png","isVerified":true}]"#,
                _ => {
                    return Err(gem_client::ClientError::Http {
                        status: 404,
                        body: path.as_bytes().to_vec(),
                    });
                }
            };
            Ok(body.as_bytes().to_vec())
        });
        let client = JupiterClient::new_with_client(client);

        let verified = client.get_verified_tokens().await.unwrap();
        let trending = client.get_top_trending_tokens("24h", 10).await.unwrap();

        assert_eq!(verified[0].id, "So11111111111111111111111111111111111111112");
        assert!(verified[0].is_verified());
        assert_eq!(trending[0].id, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN");
        assert_eq!(trending[0].icon.as_deref(), Some("https://example.com/jup.png"));
    }
}
