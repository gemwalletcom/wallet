use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType, Target, build_path_with_query};

use crate::models::transaction::PaymentsQuery;

#[derive(Clone, Debug)]
pub enum HorizonTarget {
    GetNodeStatus,
    GetFees,
    GetTransaction { hash: String },
    GetAssets { issuer: String, limit: usize },
    GetAccount { address: String },
    GetAccountPayments { address: String, query: PaymentsQuery },
    GetTransactionPayments { hash: String, query: PaymentsQuery },
    GetLedgerPayments { ledger: u64, query: PaymentsQuery },
    SubmitTransaction,
}

impl Target for HorizonTarget {
    fn path(&self) -> String {
        match self {
            Self::GetNodeStatus => "/".to_string(),
            Self::GetFees => "/fee_stats".to_string(),
            Self::GetTransaction { hash } => format!("/transactions/{hash}"),
            Self::GetAssets { issuer, limit } => format!("/assets?asset_issuer={issuer}&limit={limit}"),
            Self::GetAccount { address } => format!("/accounts/{address}"),
            Self::GetAccountPayments { address, query } => build_path_with_query(&format!("/accounts/{address}/payments"), query),
            Self::GetTransactionPayments { hash, query } => build_path_with_query(&format!("/transactions/{hash}/payments"), query),
            Self::GetLedgerPayments { ledger, query } => build_path_with_query(&format!("/ledgers/{ledger}/payments"), query),
            Self::SubmitTransaction => "/transactions_async".to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::SubmitTransaction => HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationFormUrlEncoded.as_str().to_string())]),
            _ => HashMap::new(),
        }
    }
}
