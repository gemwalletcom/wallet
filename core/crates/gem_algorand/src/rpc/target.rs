use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType, Target};

#[derive(Clone, Debug)]
pub enum AlgorandTarget {
    GetAccount { address: String },
    GetAsset { asset_id: String },
    GetTransactionsParams,
    GetPendingTransaction { transaction_id: String },
    SendTransaction,
}

impl Target for AlgorandTarget {
    fn path(&self) -> String {
        match self {
            Self::GetAccount { address } => format!("/v2/accounts/{address}"),
            Self::GetAsset { asset_id } => format!("/v2/assets/{asset_id}"),
            Self::GetTransactionsParams => "/v2/transactions/params".to_string(),
            Self::GetPendingTransaction { transaction_id } => format!("/v2/transactions/pending/{transaction_id}"),
            Self::SendTransaction => "/v2/transactions".to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::SendTransaction => HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationXBinary.as_str().to_string())]),
            _ => HashMap::new(),
        }
    }
}
