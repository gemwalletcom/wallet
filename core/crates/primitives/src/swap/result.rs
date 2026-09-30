use super::SwapStatus;
use crate::{Chain, Transaction, TransactionSwapMetadata};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResult {
    pub status: SwapStatus,
    pub metadata: Option<TransactionSwapMetadata>,
    pub eta_in_seconds: Option<u32>,
}

impl SwapResult {
    pub fn pending() -> Self {
        Self {
            status: SwapStatus::Pending,
            metadata: None,
            eta_in_seconds: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapResultRequest {
    pub chain: Chain,
    pub transaction_hash: String,
    pub deposit_address: Option<String>,
    pub deposit_memo: Option<String>,
}

impl SwapResultRequest {
    pub fn new(chain: Chain, transaction_hash: &str) -> Self {
        Self {
            chain,
            transaction_hash: transaction_hash.to_string(),
            deposit_address: None,
            deposit_memo: None,
        }
    }
}

impl From<&Transaction> for SwapResultRequest {
    fn from(transaction: &Transaction) -> Self {
        Self {
            chain: transaction.id.chain,
            transaction_hash: transaction.id.hash.clone(),
            deposit_address: Some(transaction.to.clone()).filter(|address| !address.is_empty()),
            deposit_memo: transaction.memo.clone().filter(|memo| !memo.is_empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_result_request_from_transaction() {
        let transaction = Transaction {
            to: "deposit".to_string(),
            memo: Some("memo".to_string()),
            ..Transaction::mock()
        };
        let request = SwapResultRequest::from(&transaction);
        assert_eq!(request.chain, transaction.id.chain);
        assert_eq!(request.transaction_hash, transaction.id.hash);
        assert_eq!(request.deposit_address.as_deref(), Some("deposit"));
        assert_eq!(request.deposit_memo.as_deref(), Some("memo"));

        let request = SwapResultRequest::from(&Transaction {
            to: String::new(),
            memo: Some(String::new()),
            ..Transaction::mock()
        });
        assert_eq!(request.deposit_address, None);
        assert_eq!(request.deposit_memo, None);
    }
}
