use super::{config::CoinMarketCapProviderConfig, mapper::is_native_token, model::AssetImage};
use coinmarketcap::{CoinMarketCapClient, Info, get_chain_for_coinmarketcap_platform, get_coinmarketcap_logo_url};
use std::error::Error;

pub struct CoinMarketCapProvider {
    client: CoinMarketCapClient,
    config: CoinMarketCapProviderConfig,
}

impl CoinMarketCapProvider {
    pub fn new(client: CoinMarketCapClient, config: CoinMarketCapProviderConfig) -> Self {
        Self { client, config }
    }

    pub async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let ids = self
            .client
            .get_latest_listings(self.config.top_count)
            .await?
            .into_iter()
            .filter(coinmarketcap::Listing::is_token)
            .map(|listing| listing.id)
            .collect();
        self.get_asset_images_by_ids(ids).await
    }

    pub async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let ids = self
            .client
            .get_trending_latest(self.config.trending_count)
            .await?
            .into_iter()
            .filter(coinmarketcap::Listing::is_token)
            .map(|listing| listing.id)
            .collect();
        self.get_asset_images_by_ids(ids).await
    }

    pub async fn get_asset_images(&self, id_or_symbol: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_info_by_id_or_symbol(id_or_symbol).await?.into_iter().flat_map(Self::map_info).collect())
    }

    async fn get_asset_images_by_ids(&self, ids: Vec<u64>) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let mut images = Vec::new();
        for chunk in ids.chunks(100) {
            images.extend(self.client.get_info_by_ids(chunk).await?.into_iter().flat_map(Self::map_info));
        }
        Ok(images)
    }

    fn map_info(info: Info) -> Vec<AssetImage> {
        if info.logo.is_empty() || !info.is_token() {
            return vec![];
        }

        let Some(image_url) = get_coinmarketcap_logo_url(&info.logo) else {
            return vec![];
        };

        info.contract_address
            .into_iter()
            .filter_map(|contract| {
                let chain = get_chain_for_coinmarketcap_platform(&contract.platform)?;
                let image = AssetImage {
                    chain,
                    token_id: contract.contract_address,
                    image_url: image_url.clone(),
                };
                (!is_native_token(&image)).then_some(image)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coinmarketcap::{ContractAddress, Platform, PlatformCoin};

    #[test]
    fn test_map_info_skips_native_placeholders() {
        let token = Info {
            logo: "https://s2.coinmarketcap.com/static/img/coins/256x256/825.png".to_string(),
            platform: Some(Default::default()),
            contract_address: vec![
                contract("Ethereum", "ethereum", "0x0000000000000000000000000000000000000000"),
                contract("Ethereum", "ethereum", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ],
        };
        let images = CoinMarketCapProvider::map_info(token);

        assert_eq!(images.len(), 1);
        assert_eq!(images[0].token_id, "0xdAC17F958D2ee523a2206206994597C13D831ec7");
    }

    fn contract(name: &str, slug: &str, contract_address: &str) -> ContractAddress {
        ContractAddress {
            contract_address: contract_address.to_string(),
            platform: Platform {
                name: name.to_string(),
                coin: PlatformCoin { slug: slug.to_string() },
            },
        }
    }
}
