use super::{ImageListProvider, ImageProvider, config::JupiterProviderConfig, model::AssetImage};
use ::jupiter::{JupiterClient, Token};
use async_trait::async_trait;
use primitives::Chain;
use std::error::Error;

pub struct JupiterProvider {
    client: JupiterClient,
    config: JupiterProviderConfig,
}

impl JupiterProvider {
    pub fn new(client: JupiterClient, config: JupiterProviderConfig) -> Self {
        Self { client, config }
    }

    fn map_tokens(tokens: Vec<Token>) -> Vec<AssetImage> {
        tokens
            .into_iter()
            .filter(Token::is_verified)
            .filter_map(|token| {
                let image_url = token.icon?;
                Some(AssetImage {
                    chain: Chain::Solana,
                    token_id: token.id,
                    image_url,
                })
            })
            .collect()
    }
}

#[async_trait]
impl ImageProvider for JupiterProvider {
    async fn get_asset_images(&self, token_id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_verified_tokens().await?;
        Ok(Self::map_tokens(tokens).into_iter().filter(|image| image.token_id == token_id).collect())
    }
}

#[async_trait]
impl ImageListProvider for JupiterProvider {
    async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_verified_tokens().await?;
        Ok(Self::map_tokens(tokens.into_iter().take(self.config.top_count).collect()))
    }

    async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_top_trending_tokens(&self.config.trending_interval, self.config.trending_count).await?;
        Ok(Self::map_tokens(tokens))
    }
}
