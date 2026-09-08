use gem_client::{Client, ClientError, ClientExt, ReqwestClient};

use crate::{model::Pair, target::DexScreenerTarget};

#[derive(Debug, Clone)]
pub struct DexScreenerClient<C: Client> {
    client: C,
}

impl DexScreenerClient<ReqwestClient> {
    pub fn new_with_reqwest_client(client: reqwest::Client) -> Self {
        Self::new_with_client(ReqwestClient::new("https://api.dexscreener.com".to_string(), client))
    }
}

impl<C: Client> DexScreenerClient<C> {
    pub fn new_with_client(client: C) -> Self {
        Self { client }
    }

    pub async fn get_token_pairs(&self, chain: &str, address: &str) -> Result<Vec<Pair>, ClientError> {
        self.client
            .get(DexScreenerTarget::TokenPairs {
                chain: chain.to_string(),
                address: address.to_string(),
            })
            .await
    }
}
