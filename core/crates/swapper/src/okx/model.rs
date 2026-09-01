use crate::SwapperError;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone)]
pub struct OkxClientConfig {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub project: String,
}

impl std::fmt::Debug for OkxClientConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("OkxClientConfig").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OkxApiResponse<T> {
    pub code: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
}

impl<T> OkxApiResponse<T> {
    pub fn first_data(self, fallback: &str) -> Result<T, SwapperError> {
        if self.code != "0" {
            let message = if self.msg.is_empty() { fallback.to_string() } else { self.msg };
            return Err(SwapperError::ComputeQuoteError(message));
        }
        self.data.into_iter().next().ok_or(SwapperError::NoQuoteAvailable)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenInfo {
    pub token_contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QuoteData {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub to_token_amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SignatureData {
    #[serde(default)]
    pub approve_contract: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TransactionData {
    #[serde(default)]
    pub data: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub gas: String,
    #[serde(default)]
    pub signature_data: Option<Vec<String>>,
}

impl TransactionData {
    pub fn get_value(&self) -> Option<BigUint> {
        if self.value.is_empty() {
            Some(BigUint::ZERO)
        } else {
            BigUint::from_str(&self.value).ok()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteParams {
    pub chain_index: String,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub slippage_percent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex_ids: Option<String>,
    pub fee_percent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapParams {
    pub chain_index: String,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub user_wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_transaction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_slippage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auto_slippage_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex_ids: Option<String>,
    pub fee_percent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_token_referrer_wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_token_referrer_wallet_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SwapDataResult {
    pub tx: TransactionData,
}

impl OkxApiResponse<SwapDataResult> {
    pub fn swap_transaction(self) -> Result<TransactionData, SwapperError> {
        let transaction_data = self.first_data("Failed to fetch OKX swap data")?.tx;
        if transaction_data.data.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(transaction_data)
    }
}
