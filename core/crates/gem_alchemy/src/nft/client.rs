use std::collections::HashSet;
use std::error::Error;

use gem_client::{Client as Transport, ClientExt};

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
            let mut query = vec![
                ("owner".to_string(), owner.to_string()),
                ("pageSize".to_string(), page_size.to_string()),
                ("withMetadata".to_string(), false.to_string()),
            ];
            if let Some(key) = page_key {
                query.push(("pageKey".to_string(), key));
            }
            let response: OwnedNftsResponse = self.client.get_with_query("/getNFTsForOwner", &query).await?;
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
            .get_with_query("/getContractMetadata", &[("contractAddress".to_string(), contract_address.to_string())])
            .await?)
    }

    pub async fn get_nft_metadata(&self, contract_address: &str, token_id: &str) -> Result<NftMetadata, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get_with_query(
                "/getNFTMetadata",
                &[("contractAddress".to_string(), contract_address.to_string()), ("tokenId".to_string(), token_id.to_string())],
            )
            .await?)
    }
}
