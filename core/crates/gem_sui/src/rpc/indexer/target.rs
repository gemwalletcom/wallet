use gem_client::Target;
use serde::Serialize;

#[derive(Clone, Debug)]
pub enum SuiIndexerTarget {
    Transactions { address: String, limit: usize, before: Option<String> },
}

impl Target for SuiIndexerTarget {
    fn path(&self) -> String {
        "/graphql".to_string()
    }
}

impl SuiIndexerTarget {
    pub fn body(&self) -> GraphqlRequest {
        match self {
            Self::Transactions { address, limit, before } => GraphqlRequest {
                operation_name: "GetTransactionsByAddress",
                variables: TransactionsVariables {
                    address: address.clone(),
                    limit: *limit,
                    before: before.clone(),
                },
                query: self.query(),
            },
        }
    }

    pub fn query(&self) -> &'static str {
        match self {
            Self::Transactions { .. } => {
                "query GetTransactionsByAddress($address: SuiAddress!, $limit: Int!, $before: String) { transactions(last: $limit, before: $before, filter: { affectedAddress: $address }) { nodes { digest effects { status timestamp gasEffects { gasObject { owner { ... on AddressOwner { address { address } } } } gasSummary { computationCost storageCost storageRebate nonRefundableStorageFee } } balanceChanges(first: 50) { nodes { owner { address } coinType { repr } amount } } events(first: 50) { nodes { contents { type { repr } json } transactionModule { package { address } } } } } } pageInfo { hasPreviousPage startCursor } } }"
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlRequest {
    pub operation_name: &'static str,
    pub variables: TransactionsVariables,
    pub query: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionsVariables {
    pub address: String,
    pub limit: usize,
    pub before: Option<String>,
}
