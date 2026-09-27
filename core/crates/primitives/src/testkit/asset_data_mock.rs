use crate::{Account, Asset, AssetData, AssetMetaData, Balance};

impl AssetData {
    pub fn mock(asset: Asset, balance: Balance) -> Self {
        Self {
            account: Account::mock(asset.chain(), ""),
            asset,
            balance,
            price: None,
            price_alerts: vec![],
            metadata: AssetMetaData::mock(),
            associations: vec![],
        }
    }
}
