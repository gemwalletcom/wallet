use std::collections::HashSet;
use std::error::Error;

use gem_client::{Client as Transport, ClientExt};

use super::model::OwnedNftsQuery;
use super::target::AlchemyNftTarget;
use super::{ContractMetadata, NftMetadata, OwnedNft, OwnedNftsResponse};

pub struct Client<C: Transport> {
    client: C,
}

impl<C: Transport> Client<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_nfts_by_owner(&self, owner: &str, page_size: usize) -> Result<Vec<OwnedNft>, Box<dyn Error + Send + Sync>> {
        let mut assets = Vec::new();
        let mut page_key = None;
        let mut seen_page_keys = HashSet::new();

        loop {
            let query = OwnedNftsQuery {
                owner: owner.to_string(),
                page_size,
                with_metadata: false,
                page_key,
            };
            let response: OwnedNftsResponse = self.client.get(AlchemyNftTarget::OwnedNfts { query }).await?;
            assets.extend(response.owned_nfts);

            let Some(next_page_key) = response.page_key else {
                break;
            };
            if !seen_page_keys.insert(next_page_key.clone()) {
                return Err("Alchemy NFT pagination returned a repeated page key".into());
            }
            page_key = Some(next_page_key);
        }

        Ok(assets)
    }

    pub async fn get_contract_metadata(&self, contract_address: &str) -> Result<ContractMetadata, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(AlchemyNftTarget::ContractMetadata {
                contract_address: contract_address.to_string(),
            })
            .await?)
    }

    pub async fn get_nft_metadata(&self, contract_address: &str, token_id: &str) -> Result<NftMetadata, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(AlchemyNftTarget::NftMetadata {
                contract_address: contract_address.to_string(),
                token_id: token_id.to_string(),
            })
            .await?)
    }
}
