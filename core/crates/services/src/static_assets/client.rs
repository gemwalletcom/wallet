use super::models::Validator;
use super::target::StaticAssetsTarget;
use gem_client::{Client, ClientError, ClientExt, ReqwestClient};
use primitives::{AssetId, Chain};

#[derive(Clone)]
pub struct StaticAssetsClient<C: Client = ReqwestClient> {
    client: C,
}

impl StaticAssetsClient<ReqwestClient> {
    pub fn new(url: &str) -> Self {
        Self::new_with_client(ReqwestClient::new(url.to_string(), gem_client::reqwest_client()))
    }
}

impl<C: Client> StaticAssetsClient<C> {
    pub fn new_with_client(client: C) -> Self {
        Self { client }
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<Validator>, ClientError> {
        self.client.get(StaticAssetsTarget::Validators { chain }).await
    }

    pub async fn get_assets_list(&self, chain: Chain) -> Result<Vec<AssetId>, ClientError> {
        let addresses: Vec<String> = self.client.get(StaticAssetsTarget::Assets { chain }).await?;
        Ok(addresses.into_iter().map(|x| AssetId::from(chain, Some(x))).collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_assets_list() {
        let client = StaticAssetsClient::new_with_client(MockClient::new().with_get(|path| {
            assert_eq!(path, "/blockchains/ethereum/assets.json");
            Ok(br#"["0xdAC17F958D2ee523a2206206994597C13D831ec7"]"#.to_vec())
        }));

        let assets = client.get_assets_list(Chain::Ethereum).await.unwrap();

        assert_eq!(assets, vec![AssetId::from(Chain::Ethereum, Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()))]);
    }

    #[tokio::test]
    async fn test_get_assets_list_not_found_is_error() {
        let client = StaticAssetsClient::new_with_client(MockClient::new().with_get(|_| Err(ClientError::Http { status: 404, body: vec![] })));

        let result = client.get_assets_list(Chain::Ethereum).await;

        assert!(matches!(result, Err(ClientError::Http { status: 404, .. })));
    }
}
