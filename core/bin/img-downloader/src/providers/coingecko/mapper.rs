use std::collections::HashMap;

use coingecko::{Coin, CoinMarket, get_chain_for_coingecko_platform_id, model::SearchTrending};

use crate::providers::{mapper::is_native_token, model::AssetImage};

pub(super) fn map_market_images(markets: Vec<CoinMarket>, mut coins_by_id: HashMap<String, Coin>) -> Vec<AssetImage> {
    markets
        .into_iter()
        .flat_map(|market| {
            coins_by_id
                .remove(&market.id)
                .map(|coin| map_platform_images(coin.platforms, market.image))
                .unwrap_or_default()
        })
        .collect()
}

pub(super) fn map_trending_images(trending: SearchTrending, mut coins_by_id: HashMap<String, Coin>) -> Vec<AssetImage> {
    trending
        .coins
        .into_iter()
        .flat_map(|trending_item| {
            let image_url = trending_item.item.large.unwrap_or_default();
            coins_by_id
                .remove(&trending_item.item.id)
                .map(|coin| map_platform_images(coin.platforms, image_url))
                .unwrap_or_default()
        })
        .collect()
}

pub(super) fn coins_by_id(coins: Vec<Coin>) -> HashMap<String, Coin> {
    coins.into_iter().map(|coin| (coin.id.clone(), coin)).collect()
}

pub(super) fn map_platform_images(platforms: HashMap<String, Option<String>>, image_url: String) -> Vec<AssetImage> {
    if image_url.is_empty() {
        return vec![];
    }

    platforms
        .into_iter()
        .filter(|(platform, _)| !platform.is_empty())
        .filter_map(|(platform, address)| {
            let chain = get_chain_for_coingecko_platform_id(&platform);
            let address = address?;
            let chain = chain?;
            if address.is_empty() {
                return None;
            }
            let image = AssetImage {
                chain,
                token_id: address,
                image_url: image_url.clone(),
            };
            (!is_native_token(&image)).then_some(image)
        })
        .collect()
}
