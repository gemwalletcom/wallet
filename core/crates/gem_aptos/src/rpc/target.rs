use std::collections::HashMap;

use gem_client::{CONTENT_TYPE, ContentType, build_path_with_query};

use crate::models::SimulateTransactionQuery;

#[derive(Clone, Debug)]
pub enum AptosTarget {
    GetLedger,
    GetBlock { height: u64 },
    GetAccount { address: String },
    GetAccountTransactions { address: String },
    GetAccountResource { address: String, resource: String },
    GetAccountBalance { address: String, asset_type: String },
    GetTransaction { hash: String },
    GetGasPrice,
    SimulateTransaction { query: SimulateTransactionQuery },
    SubmitTransaction,
    View,
}

impl AptosTarget {
    pub fn path(&self) -> String {
        match self {
            Self::GetLedger => "/v1/".to_string(),
            Self::GetBlock { height } => format!("/v1/blocks/by_height/{height}?with_transactions=true"),
            Self::GetAccount { address } => format!("/v1/accounts/{address}"),
            Self::GetAccountTransactions { address } => format!("/v1/accounts/{address}/transactions"),
            Self::GetAccountResource { address, resource } => format!("/v1/accounts/{address}/resource/{resource}"),
            Self::GetAccountBalance { address, asset_type } => format!("/v1/accounts/{address}/balance/{asset_type}"),
            Self::GetTransaction { hash } => format!("/v1/transactions/by_hash/{hash}"),
            Self::GetGasPrice => "/v1/estimate_gas_price".to_string(),
            Self::SimulateTransaction { query } => build_path_with_query("/v1/transactions/simulate", query),
            Self::SubmitTransaction => "/v1/transactions".to_string(),
            Self::View => "/v1/view".to_string(),
        }
    }

    pub fn headers(&self) -> HashMap<String, String> {
        match self {
            Self::SubmitTransaction => HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationAptosBcs.as_str().to_string())]),
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(AptosTarget::GetBlock { height: 42 }.path(), "/v1/blocks/by_height/42?with_transactions=true");
        assert_eq!(AptosTarget::GetAccount { address: "0x1".into() }.path(), "/v1/accounts/0x1");
        assert_eq!(AptosTarget::GetAccountTransactions { address: "0x1".into() }.path(), "/v1/accounts/0x1/transactions");
        assert_eq!(
            AptosTarget::GetAccountResource {
                address: "0x1".into(),
                resource: "0x1::stake::ValidatorSet".into()
            }
            .path(),
            "/v1/accounts/0x1/resource/0x1::stake::ValidatorSet"
        );
        assert_eq!(
            AptosTarget::GetAccountBalance {
                address: "0x1".into(),
                asset_type: "0x1::aptos_coin::AptosCoin".into()
            }
            .path(),
            "/v1/accounts/0x1/balance/0x1::aptos_coin::AptosCoin"
        );
        assert_eq!(AptosTarget::GetTransaction { hash: "0xabc".into() }.path(), "/v1/transactions/by_hash/0xabc");
        assert_eq!(
            AptosTarget::SimulateTransaction {
                query: SimulateTransactionQuery {
                    estimate_max_gas_amount: true,
                    estimate_gas_unit_price: false,
                    estimate_prioritized_gas_unit_price: false,
                },
            }
            .path(),
            "/v1/transactions/simulate?estimate_max_gas_amount=true&estimate_gas_unit_price=false&estimate_prioritized_gas_unit_price=false"
        );
    }
}
