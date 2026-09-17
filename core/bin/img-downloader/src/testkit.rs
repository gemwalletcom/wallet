use primitives::Chain;

use crate::providers::model::AssetImage;

impl AssetImage {
    pub fn mock_with_token(chain: Chain, token_id: &str) -> Self {
        Self {
            chain,
            token_id: token_id.to_string(),
            image_url: String::new(),
        }
    }
}
