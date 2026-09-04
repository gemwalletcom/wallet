use crate::model::{PositionsResponse, Token, TokenSearchResult};
use crate::target::JupiterTarget;
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
        Ok(self.client.get(JupiterTarget::VerifiedTokens).headers(self.headers()).await?)
    }

    pub async fn get_top_trending_tokens(&self, interval: &str, limit: usize) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        let target = JupiterTarget::TopTrending {
            interval: interval.to_string(),
            limit,
        };
        Ok(self.client.get(target).headers(self.headers()).await?)
    }

    pub async fn get_token(&self, mint: &str) -> Result<Option<TokenSearchResult>, Box<dyn Error + Send + Sync>> {
        let tokens: Vec<TokenSearchResult> = self.client.get(JupiterTarget::Search { query: mint.to_string() }).headers(self.headers()).await?;
        Ok(tokens.into_iter().find(|token| token.id == mint))
    }

    pub async fn get_wallet_positions(&self, address: &str) -> Result<PositionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(JupiterTarget::Positions { address: address.to_string() }).headers(self.headers()).await?)
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
    use std::sync::{Arc, Mutex};

    type Requests = Arc<Mutex<Vec<(String, HashMap<String, String>)>>>;

    fn recording_client(response: &'static str) -> (MockClient, Requests) {
        let requests: Requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let client = MockClient::new().with_get_with_headers(move |path, headers| {
            recorded.lock().unwrap().push((path.to_string(), headers.clone()));
            Ok(response.as_bytes().to_vec())
        });
        (client, requests)
    }

    fn static_client(response: &'static str) -> JupiterClient<MockClient> {
        JupiterClient::new_with_client(MockClient::new().with_get(move |_| Ok(response.as_bytes().to_vec())))
    }

    #[tokio::test]
    async fn test_get_token_returns_only_exact_match() {
        let mint = "MintCaseSensitive";
        let client = static_client(
            r#"[
                {"id":"mintcasesensitive","isVerified":true,"audit":null},
                {"id":"MintCaseSensitive","isVerified":false,"audit":null}
            ]"#,
        );
        let token = client.get_token(mint).await.unwrap().unwrap();

        assert_eq!(token.id, mint);
        assert_eq!(token.is_verified, Some(false));

        let wrong_id_client = static_client(r#"[{"id":"mintcasesensitive","isVerified":true,"audit":null}]"#);
        let empty_client = static_client("[]");

        assert!(wrong_id_client.get_token(mint).await.unwrap().is_none());
        assert!(empty_client.get_token(mint).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_token_requests_include_api_key_and_query() {
        let (mock, requests) = recording_client(r#"[{"id":"MintCaseSensitive","icon":null,"isVerified":true,"audit":null}]"#);
        let client = JupiterClient::new_with_client_and_api_key(mock, "api-key".to_string());

        client.get_token("MintCaseSensitive").await.unwrap();
        client.get_verified_tokens().await.unwrap();
        client.get_top_trending_tokens("24h", 10).await.unwrap();

        let headers = HashMap::from([(JUPITER_API_HEADER_KEY.to_string(), "api-key".to_string())]);
        assert_eq!(
            *requests.lock().unwrap(),
            vec![
                ("/tokens/v2/search?query=MintCaseSensitive".to_string(), headers.clone()),
                ("/tokens/v2/tag?query=verified".to_string(), headers.clone()),
                ("/tokens/v2/toptrending/24h?limit=10".to_string(), headers),
            ]
        );
    }

    #[tokio::test]
    async fn test_blank_api_key_is_omitted() {
        let (mock, requests) = recording_client("[]");
        let client = JupiterClient::new_with_client_and_api_key(mock, String::new());

        client.get_token("MintCaseSensitive").await.unwrap();

        assert_eq!(requests.lock().unwrap()[0].1, HashMap::new());
    }
}
