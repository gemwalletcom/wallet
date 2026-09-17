use chrono::Utc;

use crate::{Asset, AssetAssociation, AssetAssociationType, AssetBasic, AssetFull, AssetProperties, AssetScore, Chain, Price, PriceProvider};

impl AssetAssociation {
    pub fn mock() -> Self {
        Self {
            asset_id: Asset::mock_eth().id,
            association_type: AssetAssociationType::Official,
        }
    }
}

impl AssetFull {
    pub fn mock() -> Self {
        let asset = Asset::mock_btc();
        Self {
            properties: AssetProperties::default(asset.id.clone()),
            asset,
            score: AssetScore::default(),
            tags: vec![],
            links: vec![],
            associations: vec![],
            perpetuals: vec![],
            price: None,
            market: None,
        }
    }
}

impl AssetBasic {
    pub fn mock_with_price(chain: Chain, price: f64, price_change_percentage_24h: f64) -> Self {
        Self {
            price: Some(Price::new(price, price_change_percentage_24h, Utc::now(), PriceProvider::Coingecko)),
            ..Asset::from_chain(chain).as_basic_primitive()
        }
    }
}
