use serde::Deserialize;

use crate::models::token::TokenBalance;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateTransactionResult {
    pub err: Option<serde_json::Value>,
    #[serde(default)]
    pub pre_balances: Vec<u64>,
    #[serde(default)]
    pub post_balances: Vec<u64>,
    pub pre_token_balances: Option<Vec<TokenBalance>>,
    pub post_token_balances: Option<Vec<TokenBalance>>,
}
