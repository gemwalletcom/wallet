use std::error::Error;

use gem_client::{Client, ClientExt};
use primitives::Chain;

use super::model::{Collection, Contract, NftResponse, NftsResponse};

pub struct OpenSeaClient<C: Client> {
    client: C,
}

impl<C: Client> OpenSeaClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    fn chain_id(chain: Chain) -> Result<&'static str, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::Ethereum => Ok("ethereum"),
            Chain::Polygon => Ok("polygon"),
            _ => Err(format!("Unsupported chain for OpenSea: {:?}", chain).into()),
        }
    }

    pub async fn get_nfts_by_account(&self, chain: Chain, account_address: &str) -> Result<NftsResponse, Box<dyn Error + Send + Sync>> {
        let path = format!("/api/v2/chain/{}/account/{}/nfts", Self::chain_id(chain)?, account_address);
        Ok(self.client.get_with_query(&path, &[("limit".to_string(), "100".to_string())]).await?)
    }

    pub async fn get_collection_by_contract(&self, chain: Chain, contract_address: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let contract = self.get_contract(chain, contract_address).await?;
        self.get_collection_by_slug(&contract.collection).await
    }

    pub async fn get_contract(&self, chain: Chain, contract_address: &str) -> Result<Contract, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/chain/{}/contract/{}", Self::chain_id(chain)?, contract_address)).await?)
    }

    pub async fn get_asset_id(&self, chain: Chain, contract_address: &str, token_id: &str) -> Result<NftResponse, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(&format!("/api/v2/chain/{}/contract/{}/nfts/{}", Self::chain_id(chain)?, contract_address, token_id))
            .await?)
    }

    pub async fn get_collection_by_slug(&self, collection_slug: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/collections/{collection_slug}")).await?)
    }
}
