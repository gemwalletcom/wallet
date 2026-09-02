use crate::model::{PositionsResponse, Token, TokenSearchResult};
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
        Ok(self
            .client
            .get_with("/tokens/v2/tag", &[("query".to_string(), "verified".to_string())], self.headers())
            .await?)
    }

    pub async fn get_top_trending_tokens(&self, interval: &str, limit: usize) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        let path = format!("/tokens/v2/toptrending/{interval}");
        Ok(self.client.get_with(&path, &[("limit".to_string(), limit.to_string())], self.headers()).await?)
    }

    pub async fn get_token(&self, mint: &str) -> Result<Option<TokenSearchResult>, Box<dyn Error + Send + Sync>> {
        let tokens: Vec<TokenSearchResult> = self
            .client
            .get_with("/tokens/v2/search", &[("query".to_string(), mint.to_string())], self.headers())
            .await?;
        Ok(tokens.into_iter().find(|token| token.id == mint))
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
    use async_trait::async_trait;
    use gem_client::{ClientError, Response, deserialize_response};
    use serde::{Serialize, de::DeserializeOwned};
    use std::{
        fmt::{Debug, Formatter},
        sync::{Arc, Mutex},
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct GetRequest {
        path: String,
        query: Vec<(String, String)>,
        headers: HashMap<String, String>,
    }

    #[derive(Clone)]
    struct MockClient {
        response: Arc<Vec<u8>>,
        requests: Arc<Mutex<Vec<GetRequest>>>,
    }

    impl MockClient {
        fn new(response: &str) -> Self {
            Self {
                response: Arc::new(response.as_bytes().to_vec()),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn requests(&self) -> Vec<GetRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl Debug for MockClient {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct("MockClient").finish()
        }
    }

    #[async_trait]
    impl Client for MockClient {
        async fn get_with<R>(&self, path: &str, query: &[(String, String)], headers: HashMap<String, String>) -> Result<R, ClientError>
        where
            R: DeserializeOwned,
        {
            self.requests.lock().unwrap().push(GetRequest {
                path: path.to_string(),
                query: query.to_vec(),
                headers,
            });
            deserialize_response(&Response {
                status: Some(200),
                data: self.response.as_ref().clone(),
            })
        }

        async fn get_url<R>(&self, url: &str) -> Result<R, ClientError>
        where
            R: DeserializeOwned,
        {
            self.get_with(url, &[], HashMap::new()).await
        }

        async fn post_with<T, R>(&self, _path: &str, _body: &T, _headers: HashMap<String, String>) -> Result<R, ClientError>
        where
            T: Serialize + Send + Sync,
            R: DeserializeOwned,
        {
            Err(ClientError::Http { status: 405, body: Vec::new() })
        }
    }

    #[tokio::test]
    async fn test_get_token_returns_only_exact_match() {
        let mint = "MintCaseSensitive";
        let client = JupiterClient::new_with_client(MockClient::new(
            r#"[
                {"id":"mintcasesensitive","isVerified":true,"audit":null},
                {"id":"MintCaseSensitive","isVerified":false,"audit":null}
            ]"#,
        ));
        let token = client.get_token(mint).await.unwrap().unwrap();

        assert_eq!(token.id, mint);
        assert_eq!(token.is_verified, Some(false));

        let wrong_id_client = JupiterClient::new_with_client(MockClient::new(r#"[{"id":"mintcasesensitive","isVerified":true,"audit":null}]"#));
        let empty_client = JupiterClient::new_with_client(MockClient::new("[]"));

        assert!(wrong_id_client.get_token(mint).await.unwrap().is_none());
        assert!(empty_client.get_token(mint).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_token_requests_include_api_key_and_query() {
        let response = r#"[{"id":"MintCaseSensitive","icon":null,"isVerified":true,"audit":null}]"#;
        let mock = MockClient::new(response);
        let client = JupiterClient::new_with_client_and_api_key(mock.clone(), "api-key".to_string());

        client.get_token("MintCaseSensitive").await.unwrap();
        client.get_verified_tokens().await.unwrap();
        client.get_top_trending_tokens("24h", 10).await.unwrap();

        let headers = HashMap::from([(JUPITER_API_HEADER_KEY.to_string(), "api-key".to_string())]);
        assert_eq!(
            mock.requests(),
            vec![
                GetRequest {
                    path: "/tokens/v2/search".to_string(),
                    query: vec![("query".to_string(), "MintCaseSensitive".to_string())],
                    headers: headers.clone(),
                },
                GetRequest {
                    path: "/tokens/v2/tag".to_string(),
                    query: vec![("query".to_string(), "verified".to_string())],
                    headers: headers.clone(),
                },
                GetRequest {
                    path: "/tokens/v2/toptrending/24h".to_string(),
                    query: vec![("limit".to_string(), "10".to_string())],
                    headers,
                },
            ]
        );
    }

    #[tokio::test]
    async fn test_blank_api_key_is_omitted() {
        let mock = MockClient::new("[]");
        let client = JupiterClient::new_with_client_and_api_key(mock.clone(), String::new());

        client.get_token("MintCaseSensitive").await.unwrap();

        assert_eq!(mock.requests()[0].headers, HashMap::new());
    }
}
