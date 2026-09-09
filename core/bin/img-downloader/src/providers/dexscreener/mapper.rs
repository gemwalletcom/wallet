use std::{cmp::Ordering, collections::HashSet};

use chain_primitives::format_token_id;
use dexscreener::{Pair, chain_from_id};
use primitives::AssetId;

use crate::providers::{mapper::is_native_token, model::AssetImage};

struct RankedImage {
    image: AssetImage,
    liquidity: Option<f64>,
    volume: f64,
}

impl RankedImage {
    fn compare_liquidity(&self, other: &Self) -> Ordering {
        match (self.liquidity, other.liquidity) {
            (Some(left), Some(right)) => left.total_cmp(&right),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }
}

fn map_pair_image(pair: Pair) -> Option<AssetImage> {
    let chain = chain_from_id(&pair.chain_id)?;
    let image = AssetImage {
        chain,
        token_id: format_token_id(chain, pair.base_token.address)?,
        image_url: pair.info?.image_url.filter(|url| !url.trim().is_empty())?,
    };
    (!is_native_token(&image)).then_some(image)
}

pub(super) fn map_asset_image(asset_id: &AssetId, pairs: Vec<Pair>) -> Option<AssetImage> {
    let token_id = format_token_id(asset_id.chain, asset_id.token_id.clone()?)?;
    pairs
        .into_iter()
        .filter_map(map_pair_image)
        .find(|image| image.chain == asset_id.chain && image.token_id == token_id)
}

pub(super) fn map_ranked_images(pairs: Vec<Pair>, count: usize) -> Vec<AssetImage> {
    let mut ranked: Vec<RankedImage> = pairs
        .into_iter()
        .filter_map(|pair| {
            let liquidity = pair.liquidity.as_ref().and_then(|liquidity| liquidity.usd);
            let volume = pair.volume.as_ref()?.h24?;
            Some(RankedImage {
                image: map_pair_image(pair)?,
                liquidity,
                volume,
            })
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.compare_liquidity(a)
            .then_with(|| b.volume.total_cmp(&a.volume))
            .then_with(|| a.image.image_url.cmp(&b.image.image_url))
    });
    let mut assets = HashSet::new();
    ranked.retain(|candidate| assets.insert((candidate.image.chain, candidate.image.token_id.clone())));
    ranked.sort_by(|a, b| {
        b.volume
            .total_cmp(&a.volume)
            .then_with(|| b.compare_liquidity(a))
            .then_with(|| a.image.chain.cmp(&b.image.chain))
            .then_with(|| a.image.token_id.cmp(&b.image.token_id))
    });
    ranked.into_iter().take(count).map(|candidate| candidate.image).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_map_asset_image_matches_chain_and_base_token() {
        let pairs: Vec<Pair> = serde_json::from_str(include_str!("../../../testdata/dexscreener_pairs.json")).unwrap();
        let asset_id = AssetId::new("robinhood_0x15d36b6a28d8327abc7afabf0f106ae2c9af5c4d").unwrap();
        assert_eq!(
            map_asset_image(&asset_id, pairs),
            Some(AssetImage {
                chain: Chain::Robinhood,
                token_id: "0x15d36B6A28d8327ABc7aFABF0F106AE2c9Af5C4d".to_string(),
                image_url: "https://example.com/pare.png".to_string()
            })
        );
        assert_eq!(map_asset_image(&asset_id, vec![]), None);
    }

    #[test]
    fn test_map_asset_image_preserves_solana_case() {
        let pairs = serde_json::from_str(include_str!("../../../testdata/dexscreener_solana_pairs.json")).unwrap();
        assert_eq!(map_asset_image(&AssetId::token(Chain::Solana, "token"), pairs), None);
    }

    #[test]
    fn test_ranked_images_use_daily_volume_without_minimums() {
        let pairs: Vec<Pair> = serde_json::from_str(include_str!("../../../testdata/dexscreener_ranked_pairs.json")).unwrap();
        let ranked = map_ranked_images(pairs.clone(), 50);
        assert_eq!(
            ranked.iter().map(|image| image.token_id.as_str()).collect::<Vec<_>>(),
            vec![
                "0x0000000000000000000000000000000000000004",
                "0x0000000000000000000000000000000000000007",
                "0x0000000000000000000000000000000000000005",
                "0x0000000000000000000000000000000000000002",
                "0x0000000000000000000000000000000000000001",
                "0x0000000000000000000000000000000000000003",
                "0x0000000000000000000000000000000000000006",
            ]
        );
        assert_eq!(map_ranked_images(pairs.iter().rev().cloned().collect(), 50), ranked);
        assert_eq!(map_ranked_images(pairs.clone(), 2), ranked[..2]);
        assert_eq!(map_ranked_images(pairs.clone(), 0), vec![]);

        let expected = map_asset_image(&AssetId::token(Chain::Base, "0x0000000000000000000000000000000000000001"), vec![pairs[0].clone()]).unwrap();
        let mut without_liquidity = pairs[0].clone();
        without_liquidity.liquidity = None;
        without_liquidity.volume.as_mut().unwrap().h24 = Some(0.0);
        assert_eq!(map_ranked_images(vec![without_liquidity], 50), vec![expected]);

        let mut missing_daily_volume = pairs[0].clone();
        missing_daily_volume.volume.as_mut().unwrap().h24 = None;
        assert_eq!(map_ranked_images(vec![missing_daily_volume], 50), vec![]);
    }
}
