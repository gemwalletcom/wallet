use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FungibleAssetActivity {
    pub transaction_version: u64,
    pub asset_type: String,
    pub amount: u64,
}

#[derive(Debug, Deserialize)]
pub struct FungibleAssetActivities {
    pub fungible_asset_activities: Vec<FungibleAssetActivity>,
}
