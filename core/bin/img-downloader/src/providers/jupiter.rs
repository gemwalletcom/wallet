use super::{config::JupiterProviderConfig, model::AssetImage};
use ::jupiter::{JupiterClient, Token};
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

    pub async fn get_verified_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_verified_tokens().await?;
        Ok(Self::map_tokens(tokens.into_iter().take(self.config.top_count).collect()))
    }

    pub async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_top_trending_tokens(&self.config.trending_interval, self.config.trending_count).await?;
        Ok(Self::map_tokens(tokens))
    }

    pub async fn get_verified_asset_images_by_id(&self, token_id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_verified_tokens().await?;
        Ok(Self::map_tokens(tokens).into_iter().filter(|image| image.token_id == token_id).collect())
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
