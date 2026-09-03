use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType};

#[derive(Clone, Debug)]
pub enum AlgorandTarget {
    GetAccount { address: String },
    GetAsset { asset_id: String },
    GetTransactionsParams,
    GetPendingTransaction { transaction_id: String },
    SendTransaction,
}

impl AlgorandTarget {
    pub fn path(&self) -> String {
        match self {
            Self::GetAccount { address } => format!("/v2/accounts/{address}"),
            Self::GetAsset { asset_id } => format!("/v2/assets/{asset_id}"),
            Self::GetTransactionsParams => "/v2/transactions/params".to_string(),
            Self::GetPendingTransaction { transaction_id } => format!("/v2/transactions/pending/{transaction_id}"),
            Self::SendTransaction => "/v2/transactions".to_string(),
        }
    }

    pub fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::SendTransaction => HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationXBinary.as_str().to_string())]),
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(AlgorandTarget::GetAccount { address: "ADDRESS".into() }.path(), "/v2/accounts/ADDRESS");
        assert_eq!(AlgorandTarget::GetAsset { asset_id: "31566704".into() }.path(), "/v2/assets/31566704");
        assert_eq!(
            AlgorandTarget::GetPendingTransaction { transaction_id: "TXID".into() }.path(),
            "/v2/transactions/pending/TXID"
        );
    }
}
