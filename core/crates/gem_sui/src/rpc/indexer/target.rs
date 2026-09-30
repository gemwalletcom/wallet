use gem_client::Target;
use serde::Serialize;

const TRANSACTION_FIELDS: &str = "digest sender { address } effects { status timestamp gasEffects { gasObject { owner { ... on AddressOwner { address { address } } } } gasSummary { computationCost storageCost storageRebate nonRefundableStorageFee } } balanceChanges(first: 50) { nodes { owner { address } coinType { repr } amount } } events(first: 50) { nodes { contents { type { repr } json } transactionModule { package { address } } } } }";

#[derive(Clone, Debug)]
pub enum SuiIndexerTarget {
    Transactions { address: String, limit: usize, before: Option<String> },
    Transaction { digest: String },
}

impl Target for SuiIndexerTarget {
    fn path(&self) -> String {
        String::new()
    }
}

impl SuiIndexerTarget {
    pub fn body(&self) -> GraphqlRequest {
        match self {
            Self::Transactions { address, limit, before } => GraphqlRequest {
                operation_name: "GetTransactionsByAddress",
                variables: GraphqlVariables::Transactions(TransactionsVariables {
                    address: address.clone(),
                    limit: *limit,
                    before: before.clone(),
                }),
                query: self.query(),
            },
            Self::Transaction { digest } => GraphqlRequest {
                operation_name: "GetTransaction",
                variables: GraphqlVariables::Transaction(TransactionVariables { digest: digest.clone() }),
                query: self.query(),
            },
        }
    }

    pub fn query(&self) -> String {
        match self {
            Self::Transactions { .. } => format!(
                "query GetTransactionsByAddress($address: SuiAddress!, $limit: Int!, $before: String) {{ transactions(last: $limit, before: $before, filter: {{ affectedAddress: $address }}) {{ nodes {{ {TRANSACTION_FIELDS} }} pageInfo {{ hasPreviousPage startCursor }} }} }}"
            ),
            Self::Transaction { .. } => format!("query GetTransaction($digest: String!) {{ transaction(digest: $digest) {{ {TRANSACTION_FIELDS} }} }}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlRequest {
    pub operation_name: &'static str,
    pub variables: GraphqlVariables,
    pub query: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum GraphqlVariables {
    Transactions(TransactionsVariables),
    Transaction(TransactionVariables),
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionsVariables {
    pub address: String,
    pub limit: usize,
    pub before: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionVariables {
    pub digest: String,
}
