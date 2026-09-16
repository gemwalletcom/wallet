use ::jupiter::Token;
use primitives::Chain;

use crate::providers::model::AssetImage;

pub(super) fn map_tokens(tokens: Vec<Token>) -> Vec<AssetImage> {
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
