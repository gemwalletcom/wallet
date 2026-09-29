use gem_client::Target;
use serde::Serialize;

const TRANSACTION_FIELDS: &str = "digest sender { address } effects { status timestamp gasEffects { gasObject { owner { ... on AddressOwner { address { address } } } } gasSummary { computationCost storageCost storageRebate nonRefundableStorageFee } } balanceChanges(first: 50) { nodes { owner { address } coinType { repr } amount } } events(first: 50) { nodes { contents { type { repr } json } transactionModule { package { address } } } } }";

#[derive(Clone, Debug)]
pub enum SuiIndexerTarget {
    Transactions { address: String, limit: usize, before: Option<String> },
    TransactionsAfter { address: String, limit: usize, after: Option<String> },
}

impl Target for SuiIndexerTarget {
    fn path(&self) -> String {
        String::new()
    }
}

impl SuiIndexerTarget {
    pub fn body(&self) -> GraphqlRequest {
        let (operation_name, variables) = match self {
            Self::Transactions { address, limit, before } => (
                "GetTransactionsByAddress",
                TransactionsVariables {
                    address: address.clone(),
                    limit: *limit,
                    before: before.clone(),
                    after: None,
                },
            ),
            Self::TransactionsAfter { address, limit, after } => (
                "GetTransactionsByAddressAfter",
                TransactionsVariables {
                    address: address.clone(),
                    limit: *limit,
                    before: None,
                    after: after.clone(),
                },
            ),
        };
        GraphqlRequest {
            operation_name,
            variables,
            query: self.query(),
        }
    }

    pub fn query(&self) -> String {
        match self {
            Self::Transactions { .. } => format!(
                "query GetTransactionsByAddress($address: SuiAddress!, $limit: Int!, $before: String) {{ transactions(last: $limit, before: $before, filter: {{ affectedAddress: $address }}) {{ nodes {{ {TRANSACTION_FIELDS} }} pageInfo {{ hasPreviousPage startCursor }} }} }}"
            ),
            Self::TransactionsAfter { .. } => format!(
                "query GetTransactionsByAddressAfter($address: SuiAddress!, $limit: Int!, $after: String) {{ transactions(first: $limit, after: $after, filter: {{ affectedAddress: $address }}) {{ nodes {{ {TRANSACTION_FIELDS} }} pageInfo {{ hasNextPage endCursor }} }} }}"
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlRequest {
    pub operation_name: &'static str,
    pub variables: TransactionsVariables,
    pub query: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionsVariables {
    pub address: String,
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}
