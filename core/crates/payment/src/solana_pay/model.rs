use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct TransactionRequestInfo {
    pub label: String,
    pub icon: String,
}

#[derive(Debug, Serialize)]
pub(super) struct TransactionRequest<'a> {
    pub account: &'a str,
}

#[derive(Debug, Deserialize)]
pub(super) struct TransactionResponse {
    pub transaction: String,
}
