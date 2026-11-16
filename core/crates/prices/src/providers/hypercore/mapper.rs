use std::collections::HashMap;

use chrono::Utc;
use gem_hypercore::models::metadata::perpetual_coin;
use gem_hypercore::models::spot::{SpotAssetContext, SpotMetaAndAssetContexts};
use gem_hypercore::models::token::is_spot_token_id;
use primitives::{AssetId, Chain, PerpetualData, PriceProvider, asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID};

use crate::{AssetPriceFull, AssetPriceMapping};

pub fn price_mapping(asset_id: &AssetId) -> Option<AssetPriceMapping> {
    let token_id = asset_id.token_id.as_ref()?;
    (asset_id.chain == Chain::HyperCore && (is_spot_token_id(token_id) || perpetual_coin(asset_id).is_some())).then(|| AssetPriceMapping::new(asset_id.clone(), token_id.clone()))
}

pub fn map_spot_prices(response: SpotMetaAndAssetContexts) -> Vec<AssetPriceFull> {
    let SpotMetaAndAssetContexts(meta, contexts) = response;
    let Some(usdc) = meta.tokens.iter().find(|token| token.asset_id(Chain::HyperCore) == *HYPERCORE_SPOT_USDC_ASSET_ID).map(|token| token.index) else {
        return vec![];
    };
    let tokens: HashMap<i32, AssetId> = meta.tokens.iter().map(|token| (token.index, token.asset_id(Chain::HyperCore))).collect();
    let contexts: HashMap<&str, &SpotAssetContext> = contexts.iter().map(|context| (context.coin.as_str(), context)).collect();

    meta.universe
        .iter()
        .filter_map(|market| {
            let [base, quote] = market.tokens.as_slice() else {
                return None;
            };
            if *quote != usdc {
                return None;
            }
            let context = contexts.get(market.name.as_str())?;
            let price = context.mid_px.unwrap_or(context.mark_px);
            map_price(price_mapping(tokens.get(base)?)?, price, price_change_24h(price, context.prev_day_px))
        })
        .collect()
}

pub fn map_perpetual_prices(data: Vec<PerpetualData>) -> Vec<AssetPriceFull> {
    data.into_iter()
        .filter_map(|data| map_price(price_mapping(&data.asset.id)?, data.perpetual.price, Some(data.perpetual.price_percent_change_24h)))
        .collect()
}

fn map_price(mapping: AssetPriceMapping, price: f64, price_change_percentage_24h: Option<f64>) -> Option<AssetPriceFull> {
    (price > 0.0).then(|| AssetPriceFull::new(mapping, price, price_change_percentage_24h, Utc::now(), PriceProvider::Hypercore, None))
}

fn price_change_24h(price: f64, prev_day_price: f64) -> Option<f64> {
    (prev_day_price > 0.0).then(|| (price - prev_day_price) / prev_day_price * 100.0)
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::{HYPERCORE_SPOT_HYPE_ASSET_ID, HYPERCORE_SPOT_UBTC_ASSET_ID, HYPERCORE_SPOT_UBTC_TOKEN_ID};

    use super::*;

    #[test]
    fn test_price_mapping() {
        let perpetual = AssetId::from_token(Chain::HyperCore, "perpetual::xyz:BRENTOIL");

        assert_eq!(
            price_mapping(&HYPERCORE_SPOT_UBTC_ASSET_ID),
            Some(AssetPriceMapping::new(HYPERCORE_SPOT_UBTC_ASSET_ID.clone(), HYPERCORE_SPOT_UBTC_TOKEN_ID.to_string()))
        );
        assert_eq!(price_mapping(&perpetual), Some(AssetPriceMapping::new(perpetual.clone(), "perpetual::xyz:BRENTOIL".to_string())));
        assert_eq!(price_mapping(&Chain::HyperCore.as_asset_id()), None);
        assert_eq!(price_mapping(&AssetId::from_token(Chain::HyperCore, "USDC")), None);
        assert_eq!(price_mapping(&AssetId::from_token(Chain::Ethereum, HYPERCORE_SPOT_UBTC_TOKEN_ID)), None);
    }

    #[test]
    fn test_map_spot_prices() {
        let response: SpotMetaAndAssetContexts = serde_json::from_str(include_str!("../../../testdata/hypercore/spot_meta_and_asset_ctxs.json")).unwrap();

        let prices: Vec<(AssetId, f64, Option<f64>, PriceProvider)> = map_spot_prices(response)
            .into_iter()
            .map(|price| (price.mapping.asset_id, price.price, price.price_change_percentage_24h, price.provider))
            .collect();

        assert_eq!(
            prices,
            vec![
                (AssetId::from_token(Chain::HyperCore, "PURR::0xc1fb593aeffbeb02f85e0308e9956a90::1"), 0.093666, Some(5.967802151802788), PriceProvider::Hypercore),
                (HYPERCORE_SPOT_HYPE_ASSET_ID.clone(), 69.9605, Some(2.881575270951885), PriceProvider::Hypercore),
                (HYPERCORE_SPOT_UBTC_ASSET_ID.clone(), 65341.5, Some(0.30317450571042615), PriceProvider::Hypercore),
                (AssetId::from_token(Chain::HyperCore, "USDH::0x54e00a5988577cb0b0c9ab0cb6ef7f4b::360"), 0.995445, Some(0.033664619991757114), PriceProvider::Hypercore),
            ]
        );
    }
}
