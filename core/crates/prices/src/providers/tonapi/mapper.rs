use gem_ton::Address;
use primitives::{Asset, AssetId, Chain, PriceProvider};

use crate::{AssetPriceFull, AssetPriceMapping};

use super::model::RatesResponse;
use super::stonfi_model::{StonfiAsset, StonfiAssetKind};

const EXCLUDED_ASSET_TAGS: &[&str] = &[
    "asset:blacklisted",
    "asset:deprecated",
    "asset:dmca_complaint",
    "asset:fake",
    "asset:honeypot",
    "asset:non_searchable",
    "asset:suspicious",
];

pub fn mapping_for_asset_id(asset_id: &AssetId) -> Option<AssetPriceMapping> {
    if asset_id.chain != Chain::Ton {
        return None;
    }
    if asset_id.is_native() {
        return Some(AssetPriceMapping::new(asset_id.clone(), Asset::from_chain(Chain::Ton).symbol.to_lowercase()));
    }
    mapping_for_price_id(asset_id.token_id.as_deref()?)
}

pub fn mapping_for_price_id(provider_price_id: &str) -> Option<AssetPriceMapping> {
    let native_symbol = Asset::from_chain(Chain::Ton).symbol;
    if provider_price_id.eq_ignore_ascii_case(&native_symbol) {
        let asset_id = AssetId::from_chain(Chain::Ton);
        return Some(AssetPriceMapping::new(asset_id, native_symbol.to_lowercase()));
    }
    let address = Address::parse(provider_price_id).ok()?.encode_bounceable();
    Some(AssetPriceMapping::new(AssetId::from_token(Chain::Ton, &address), address))
}

pub fn mapping_for_stonfi_asset(asset: StonfiAsset) -> Option<AssetPriceMapping> {
    if asset.tags.iter().any(|tag| EXCLUDED_ASSET_TAGS.contains(&tag.as_str())) {
        return None;
    }
    match asset.kind {
        StonfiAssetKind::Ton => mapping_for_asset_id(&AssetId::from_chain(Chain::Ton)),
        StonfiAssetKind::Jetton => mapping_for_price_id(&asset.contract_address),
        StonfiAssetKind::Unsupported => None,
    }
}

pub fn map_price(mapping: AssetPriceMapping, response: &RatesResponse) -> Option<AssetPriceFull> {
    let rates = if mapping.asset_id.is_native() {
        response.rates.get(&Asset::from_chain(Chain::Ton).symbol)
    } else {
        response.rates.get(&mapping.provider_price_id)
    }?;
    let price = rates.prices.get("USD").copied()?;
    if !price.is_finite() || price <= 0.0 {
        return None;
    }
    let price_change_percentage_24h = rates
        .diff_24h
        .get("USD")
        .and_then(|value| value.trim_end_matches('%').replace('−', "-").parse::<f64>().ok())
        .unwrap_or_default();
    Some(AssetPriceFull::simple(mapping, price, price_change_percentage_24h, PriceProvider::TonApi))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::tonapi::model::RatesResponse;
    use crate::providers::tonapi::stonfi_model::{StonfiAssetKind, StonfiAssetsResponse};
    use primitives::asset_constants::{TON_DUST_TOKEN_ID, TON_USDT_TOKEN_ID};

    const DUST_RAW_ADDRESS: &str = "0:65aac9b5e380eae928db3c8e238d9bc0d61a9320fdc2bc7a2f6c87d6fedf9208";

    #[test]
    fn test_tonapi_mappings_and_prices() {
        let native = AssetId::from_chain(Chain::Ton);
        let native_symbol = Asset::from_chain(Chain::Ton).symbol;
        let dust = AssetId::from_token(Chain::Ton, TON_DUST_TOKEN_ID);
        let ethereum = AssetId::from_chain(Chain::Ethereum);

        assert_eq!(mapping_for_asset_id(&native).unwrap().provider_price_id, native_symbol.to_lowercase());
        assert_eq!(mapping_for_asset_id(&dust).unwrap().provider_price_id, TON_DUST_TOKEN_ID);
        assert_eq!(mapping_for_asset_id(&ethereum).map(|mapping| mapping.asset_id), None);
        assert_eq!(mapping_for_price_id(&native_symbol).unwrap().asset_id, native);
        assert_eq!(mapping_for_price_id(TON_DUST_TOKEN_ID).unwrap().asset_id, dust);
        assert_eq!(mapping_for_price_id(DUST_RAW_ADDRESS).unwrap().asset_id, dust);
        assert_eq!(mapping_for_price_id("invalid").map(|mapping| mapping.asset_id), None);

        let assets: StonfiAssetsResponse = serde_json::from_str(include_str!("../../../testdata/tonapi/stonfi_assets.json")).unwrap();
        let mut assets = assets.asset_list.into_iter();
        assert_eq!(mapping_for_stonfi_asset(assets.next().unwrap()).unwrap().asset_id, native);
        assert_eq!(
            mapping_for_stonfi_asset(assets.next().unwrap()).unwrap().asset_id,
            AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)
        );
        let excluded = StonfiAsset {
            contract_address: TON_DUST_TOKEN_ID.to_string(),
            kind: StonfiAssetKind::Jetton,
            tags: vec!["asset:fake".to_string()],
        };
        assert_eq!(mapping_for_stonfi_asset(excluded).map(|mapping| mapping.asset_id), None);

        let response: RatesResponse = serde_json::from_str(include_str!("../../../testdata/tonapi/rates.json")).unwrap();
        let native_price = map_price(mapping_for_asset_id(&native).unwrap(), &response).unwrap().price;
        assert_eq!(native_price.price, 3.42);
        assert_eq!(native_price.price_change_percentage_24h, -4.58);
        let dust_price = map_price(mapping_for_asset_id(&dust).unwrap(), &response).unwrap().price;
        assert_eq!(dust_price.price, 0.62784);
        assert_eq!(dust_price.price_change_percentage_24h, 2.14);
        let zero = AssetPriceMapping::new(dust, "ZERO".to_string());
        assert_eq!(map_price(zero, &response).map(|price| price.price.price), None);
    }
}
