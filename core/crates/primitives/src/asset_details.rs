use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetAssociation, AssetId, AssetMarket, AssetScore, LinkType, Price, perpetual::PerpetualBasic};

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFull {
    pub asset: Asset,
    pub properties: AssetProperties,
    pub score: AssetScore,
    pub tags: Vec<String>,
    pub links: Vec<AssetLink>,
    pub associations: Vec<AssetAssociation>,
    pub perpetuals: Vec<PerpetualBasic>,
    pub price: Option<Price>,
    pub market: Option<AssetMarket>,
}

impl AssetFull {
    pub fn with_rate(self, rate: f64) -> Self {
        Self {
            asset: self.asset,
            properties: self.properties,
            score: self.score,
            tags: self.tags,
            links: self.links,
            associations: self.associations,
            perpetuals: self.perpetuals,
            price: self.price.map(|p| p.with_rate(rate)),
            market: self.market.map(|m| m.with_rate(rate)),
        }
    }
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBasic {
    pub asset: Asset,
    pub properties: AssetProperties,
    pub score: AssetScore,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
}

impl AssetBasic {
    pub fn new(asset: Asset, properties: AssetProperties, score: AssetScore) -> Self {
        Self {
            asset,
            properties,
            score,
            price: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetPriceMetadata {
    pub asset: AssetBasic,
    pub price: Option<Price>,
}

impl AssetPriceMetadata {
    pub fn asset_basic_with_rate(self, rate: f64) -> AssetBasic {
        AssetBasic {
            price: self.price.map(|price| price.with_rate(rate)),
            ..self.asset
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMarketPrice {
    pub price: Option<Price>,
    pub market: Option<AssetMarket>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prices: Option<Vec<Price>>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProperties {
    pub is_enabled: bool,
    pub is_buyable: bool,
    pub is_sellable: bool,
    pub is_swapable: bool,
    pub is_stakeable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staking_apr: Option<f64>,
    pub is_earnable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earn_apr: Option<f64>,
    pub has_image: bool,
    #[typeshare(skip)]
    pub has_price: bool,
}

impl AssetProperties {
    pub fn default(asset_id: AssetId) -> Self {
        let is_enabled = asset_id.is_token() || asset_id.chain.rank() >= 0;
        let is_stakeable = is_enabled && asset_id.is_native() && asset_id.chain.is_stake_supported();
        let is_swapable = is_enabled && asset_id.chain.is_swap_supported();
        Self {
            is_enabled,
            is_buyable: false,
            is_sellable: false,
            is_swapable,
            is_stakeable,
            staking_apr: None,
            is_earnable: false,
            earn_apr: None,
            has_image: false,
            has_price: false,
        }
    }
}

#[typeshare(swift = "Sendable, Equatable, Hashable")]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetLink {
    pub name: String,
    pub url: String,
}

impl AssetLink {
    pub fn new(url: &str, link_type: LinkType) -> Self {
        Self {
            name: link_type.name(),
            url: url.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::{Asset, Chain, PriceProvider, asset_constants::TEMPO_PATHUSD_ASSET_ID};

    #[test]
    fn negative_rank_native_asset_is_disabled() {
        let properties = AssetProperties::default(AssetId::from_chain(Chain::Tempo));

        assert!(!properties.is_enabled);
        assert!(!properties.is_swapable);
        assert!(AssetProperties::default(TEMPO_PATHUSD_ASSET_ID.clone()).is_swapable);
    }

    #[test]
    fn test_asset_basic_with_rate() {
        let asset = Asset::mock_btc();
        let price = Price::new(100.0, 5.0, Utc::now(), PriceProvider::Coingecko);

        let base = AssetPriceMetadata {
            asset: AssetBasic::new(asset.clone(), AssetProperties::default(asset.id.clone()), AssetScore::default()),
            price: Some(price),
        }
        .asset_basic_with_rate(1.0);
        let converted = AssetPriceMetadata {
            asset: AssetBasic::new(asset.clone(), AssetProperties::default(asset.id.clone()), AssetScore::default()),
            price: Some(price),
        }
        .asset_basic_with_rate(2.0);
        let missing = AssetPriceMetadata {
            asset: AssetBasic::new(asset.clone(), AssetProperties::default(asset.id.clone()), AssetScore::default()),
            price: None,
        }
        .asset_basic_with_rate(2.0);

        assert_eq!(base.asset, asset);
        assert_eq!(base.price.as_ref().unwrap().price, price.price);
        assert_eq!(base.price.as_ref().unwrap().price_change_percentage_24h, price.price_change_percentage_24h);
        assert_eq!(converted.price.as_ref().unwrap().price, price.price * 2.0);
        assert_eq!(converted.price.as_ref().unwrap().price_change_percentage_24h, price.price_change_percentage_24h);
        assert!(missing.price.is_none());
    }

    #[test]
    fn test_asset_full_with_rate() {
        let association = AssetAssociation::mock();
        let result = AssetFull {
            associations: vec![association.clone()],
            ..AssetFull::mock()
        }
        .with_rate(2.0);

        assert_eq!(result.associations, vec![association]);
    }
}
