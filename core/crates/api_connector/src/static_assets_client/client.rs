use super::models::Validator;
use super::target::StaticAssetsTarget;
use gem_client::{ClientError, ClientExt, ReqwestClient};
use primitives::{AssetId, Chain};

#[derive(Clone)]
pub struct StaticAssetsClient {
    client: ReqwestClient,
}

impl StaticAssetsClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: ReqwestClient::new(url.to_string(), gem_client::reqwest_client()),
        }
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<Validator>, ClientError> {
        self.client.get(StaticAssetsTarget::Validators { chain }).await
    }

    pub async fn get_assets_list(&self, chain: Chain) -> Result<Vec<AssetId>, ClientError> {
        let addresses: Vec<String> = match self.client.get(StaticAssetsTarget::Assets { chain }).await {
            Ok(addresses) => addresses,
            Err(ClientError::Http { .. }) => Vec::new(),
            Err(error) => return Err(error),
        };
        Ok(addresses.into_iter().map(|x| AssetId::from(chain, Some(x))).collect())
    }
}
