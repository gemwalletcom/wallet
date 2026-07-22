use std::error::Error;

use gem_client::{Client, ClientExt};

use super::model::{Collection, Nft};

pub struct MagicEdenSolanaClient<C: Client> {
    client: C,
}

impl<C: Client> MagicEdenSolanaClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_nfts_by_account(&self, address: &str) -> Result<Vec<Nft>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/wallets/{address}/tokens")).await?)
    }

    pub async fn get_collection_id(&self, collection_id: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/collections/{collection_id}")).await?)
    }

    pub async fn get_asset_id(&self, token_mint: &str) -> Result<Nft, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/tokens/{token_mint}")).await?)
    }
}
