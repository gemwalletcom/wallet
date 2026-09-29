use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub signature: String,
    pub source: String,
    pub fee_payer: String,
    pub fee: u64,
    pub transaction_error: Option<serde_json::Value>,
    #[serde(default)]
    pub account_data: Vec<AccountData>,
    #[serde(default)]
    pub events: Events,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Events {
    pub swap: Option<SwapEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapEvent {
    pub native_input: Option<NativeAmount>,
    pub native_output: Option<NativeAmount>,
    #[serde(default)]
    pub token_inputs: Vec<TokenAmount>,
    #[serde(default)]
    pub token_outputs: Vec<TokenAmount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NativeAmount {
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub user_account: String,
    pub mint: String,
    pub raw_token_amount: RawTokenAmount,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTokenAmount {
    pub token_amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub account: String,
    pub native_balance_change: i64,
    #[serde(default)]
    pub token_balance_changes: Vec<TokenAmount>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TransactionsQuery {
    pub limit: usize,
    pub token_accounts: &'static str,
    pub sort_order: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_signature: Option<String>,
}
