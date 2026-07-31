use crate::{Asset, AssetAssociation, AssetAssociationType, AssetFull, AssetProperties, AssetScore};

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
