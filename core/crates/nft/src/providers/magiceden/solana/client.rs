use std::error::Error;

use gem_client::{Client, ClientExt};

use super::model::{Collection, Nft};
use super::target::MagicEdenTarget;

pub struct MagicEdenSolanaClient<C: Client> {
    client: C,
}

impl<C: Client> MagicEdenSolanaClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_nfts_by_account(&self, address: &str) -> Result<Vec<Nft>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(MagicEdenTarget::WalletTokens { address: address.to_string() }).await?)
    }

    pub async fn get_collection_id(&self, collection_id: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(MagicEdenTarget::Collection { id: collection_id.to_string() }).await?)
    }

    pub async fn get_asset_id(&self, token_mint: &str) -> Result<Nft, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(MagicEdenTarget::Token { mint: token_mint.to_string() }).await?)
    }
}
