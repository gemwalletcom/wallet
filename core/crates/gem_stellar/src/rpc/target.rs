use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType, build_path_with_query};

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
    SubmitTransaction { transaction: String },
}

impl HorizonTarget {
    pub fn path(&self) -> String {
        match self {
            Self::GetNodeStatus => "/".to_string(),
            Self::GetFees => "/fee_stats".to_string(),
            Self::GetTransaction { hash } => format!("/transactions/{hash}"),
            Self::GetAssets { issuer, limit } => format!("/assets?asset_issuer={issuer}&limit={limit}"),
            Self::GetAccount { address } => format!("/accounts/{address}"),
            Self::GetAccountPayments { address, query } => build_path_with_query(&format!("/accounts/{address}/payments"), query),
            Self::GetTransactionPayments { hash, query } => build_path_with_query(&format!("/transactions/{hash}/payments"), query),
            Self::GetLedgerPayments { ledger, query } => build_path_with_query(&format!("/ledgers/{ledger}/payments"), query),
            Self::SubmitTransaction { .. } => "/transactions_async".to_string(),
        }
    }

    pub fn body(&self) -> Option<&String> {
        match self {
            Self::SubmitTransaction { transaction } => Some(transaction),
            _ => None,
        }
    }

    pub fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::SubmitTransaction { .. } => HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationFormUrlEncoded.as_str().to_string())]),
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(HorizonTarget::GetTransaction { hash: "abc".into() }.path(), "/transactions/abc");
        assert_eq!(
            HorizonTarget::GetAssets {
                issuer: "GISSUER".into(),
                limit: 200
            }
            .path(),
            "/assets?asset_issuer=GISSUER&limit=200"
        );
        assert_eq!(HorizonTarget::GetAccount { address: "GADDRESS".into() }.path(), "/accounts/GADDRESS");
        assert_eq!(
            HorizonTarget::GetAccountPayments {
                address: "GADDRESS".into(),
                query: PaymentsQuery::latest(200)
            }
            .path(),
            "/accounts/GADDRESS/payments?order=desc&limit=200&include_failed=true&join=transactions"
        );
        assert_eq!(
            HorizonTarget::GetTransactionPayments {
                hash: "abc".into(),
                query: PaymentsQuery::default()
            }
            .path(),
            "/transactions/abc/payments?include_failed=true&join=transactions"
        );
        assert_eq!(
            HorizonTarget::GetLedgerPayments {
                ledger: 5,
                query: PaymentsQuery::page(200, Some("cursor".into()))
            }
            .path(),
            "/ledgers/5/payments?limit=200&include_failed=true&cursor=cursor&join=transactions"
        );
        assert_eq!(
            HorizonTarget::GetLedgerPayments {
                ledger: 5,
                query: PaymentsQuery::page(200, None)
            }
            .path(),
            "/ledgers/5/payments?limit=200&include_failed=true&join=transactions"
        );
    }
}
