use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FungibleAssetActivity {
    pub transaction_version: u64,
    pub owner_address: String,
    pub asset_type: String,
    pub amount: u64,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub is_transaction_success: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserTransaction {
    pub version: u64,
    pub sender: String,
    pub entry_function_id_str: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FungibleAssetActivities {
    pub fungible_asset_activities: Vec<FungibleAssetActivity>,
}

#[derive(Debug, Deserialize)]
pub struct UserTransactions {
    pub user_transactions: Vec<UserTransaction>,
}
