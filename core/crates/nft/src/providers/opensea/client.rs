use std::error::Error;

use gem_client::{Client, ClientExt};
use primitives::Chain;

use super::model::{Collection, Contract, NftResponse, NftsResponse};
use super::target::OpenSeaTarget;

const ACCOUNT_NFTS_LIMIT: usize = 100;

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
        let target = OpenSeaTarget::AccountNfts {
            chain: Self::chain_id(chain)?,
            address: account_address.to_string(),
            limit: ACCOUNT_NFTS_LIMIT,
        };
        Ok(self.client.get(target).await?)
    }

    pub async fn get_collection_by_contract(&self, chain: Chain, contract_address: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        let contract = self.get_contract(chain, contract_address).await?;
        self.get_collection_by_slug(&contract.collection).await
    }

    pub async fn get_contract(&self, chain: Chain, contract_address: &str) -> Result<Contract, Box<dyn Error + Send + Sync>> {
        let target = OpenSeaTarget::Contract {
            chain: Self::chain_id(chain)?,
            address: contract_address.to_string(),
        };
        Ok(self.client.get(target).await?)
    }

    pub async fn get_asset_id(&self, chain: Chain, contract_address: &str, token_id: &str) -> Result<NftResponse, Box<dyn Error + Send + Sync>> {
        let target = OpenSeaTarget::Nft {
            chain: Self::chain_id(chain)?,
            address: contract_address.to_string(),
            token_id: token_id.to_string(),
        };
        Ok(self.client.get(target).await?)
    }

    pub async fn get_collection_by_slug(&self, collection_slug: &str) -> Result<Collection, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(OpenSeaTarget::Collection {
                slug: collection_slug.to_string(),
            })
            .await?)
    }
}
