use std::error::Error;

use ::dexscreener::{DexScreenerClient, Pair, chain_id};
use async_trait::async_trait;
use chain_primitives::format_token_id;
use primitives::AssetId;

use super::{ImageProvider, mapper::is_native_token, model::AssetImage};

pub struct DexScreenerProvider {
    client: DexScreenerClient,
}

impl DexScreenerProvider {
    pub fn new(client: DexScreenerClient) -> Self {
        Self { client }
    }

    fn map_asset_image(asset_id: &AssetId, pairs: Vec<Pair>) -> Option<AssetImage> {
        let chain = chain_id(asset_id.chain)?;
        let token_id = format_token_id(asset_id.chain, asset_id.token_id.clone()?)?;
        pairs.into_iter().find_map(|pair| {
            if pair.chain_id != chain || format_token_id(asset_id.chain, pair.base_token.address).as_ref() != Some(&token_id) {
                return None;
            }
            let image_url = pair.info?.image_url.filter(|url| !url.trim().is_empty())?;
            let image = AssetImage {
                chain: asset_id.chain,
                token_id: token_id.clone(),
                image_url,
            };
            (!is_native_token(&image)).then_some(image)
        })
    }
}

#[async_trait]
impl ImageProvider for DexScreenerProvider {
    async fn get_asset_images(&self, id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let asset_id = AssetId::new(id).ok_or("Invalid asset ID")?;
        let chain = chain_id(asset_id.chain).ok_or("Unsupported DexScreener chain")?;
        let pairs = self.client.get_token_pairs(chain, asset_id.get_token_id()?).await?;
        Ok(Self::map_asset_image(&asset_id, pairs).into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_map_asset_image_matches_chain_and_base_token() {
        let pairs: Vec<Pair> = serde_json::from_str(include_str!("../../testdata/dexscreener_pairs.json")).unwrap();
        let asset_id = AssetId::new("robinhood_0x15d36b6a28d8327abc7afabf0f106ae2c9af5c4d").unwrap();
        assert_eq!(
            DexScreenerProvider::map_asset_image(&asset_id, pairs),
            Some(AssetImage {
                chain: Chain::Robinhood,
                token_id: "0x15d36B6A28d8327ABc7aFABF0F106AE2c9Af5C4d".to_string(),
                image_url: "https://example.com/pare.png".to_string()
            })
        );
        assert_eq!(DexScreenerProvider::map_asset_image(&asset_id, vec![]), None);
    }

    #[test]
    fn test_map_asset_image_preserves_solana_case() {
        let pairs = serde_json::from_str(include_str!("../../testdata/dexscreener_solana_pairs.json")).unwrap();
        assert_eq!(DexScreenerProvider::map_asset_image(&AssetId::token(Chain::Solana, "token"), pairs), None);
    }
}
