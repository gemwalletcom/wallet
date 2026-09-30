use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IndexerTransaction {
    pub user_transactions: Vec<IndexerUserTransaction>,
    pub fungible_asset_activities: Vec<IndexerAssetActivity>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexerUserTransaction {
    pub sender: String,
    pub entry_function_id_str: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexerAssetActivity {
    pub owner_address: Option<String>,
    pub asset_type: Option<String>,
    pub amount: Option<u64>,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub is_gas_fee: bool,
    pub is_transaction_success: bool,
}
