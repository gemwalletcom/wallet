use gem_client::Target;

use crate::models::rpc::{GraphqlRequest, GraphqlVariables};

#[derive(Clone, Debug)]
pub enum CardanoTarget {
    Tip,
    Block { number: u64 },
    AddressTransactions { address: String, limit: usize },
    Balance { address: String },
    Utxos { address: String },
    NetworkMagic,
    SubmitTransaction { transaction: String },
}

impl Target for CardanoTarget {
    fn path(&self) -> String {
        "/".to_string()
    }
}

impl CardanoTarget {
    pub fn body(&self) -> GraphqlRequest {
        GraphqlRequest {
            operation_name: self.operation(),
            variables: self.variables(),
            query: self.query(),
        }
    }

    fn operation(&self) -> Option<&'static str> {
        match self {
            Self::Tip => None,
            Self::Block { .. } => Some("GetBlockByNumber"),
            Self::AddressTransactions { .. } => Some("GetTransactionsByAddress"),
            Self::Balance { .. } => Some("GetBalance"),
            Self::Utxos { .. } => Some("UtxoSetForAddress"),
            Self::NetworkMagic => Some("GetNetworkMagic"),
            Self::SubmitTransaction { .. } => Some("SubmitTransaction"),
        }
    }

    pub fn query(&self) -> &'static str {
        match self {
            Self::Tip => "{ cardano { tip { number slotNo } } }",
            Self::Block { .. } => {
                "query GetBlockByNumber($blockNumber: Int!) { blocks(where: { number: { _eq: $blockNumber } }) { number hash forgedAt transactions { hash inputs { address value } outputs { address value } fee } } }"
            }
            Self::AddressTransactions { .. } => {
                "query GetTransactionsByAddress($address: String!, $limit: Int!) { transactions(limit: $limit, order_by: { includedAt: desc }, where: { outputs: { address: { _eq: $address } } }) { hash includedAt inputs { address value } outputs { address value } fee } }"
            }
            Self::Balance { .. } => "query GetBalance($address: String!) { utxos: utxos_aggregate(where: { address: { _eq: $address }  } ) { aggregate { sum { value } } } }",
            Self::Utxos { .. } => {
                "query UtxoSetForAddress($address: String!) { utxos(order_by: { value: desc } , where: { address: { _eq: $address }  } ) { address value txHash index tokens { quantity asset { fingerprint policyId assetName } } } }"
            }
            Self::NetworkMagic => "query GetNetworkMagic { genesis { shelley { networkMagic } } }",
            Self::SubmitTransaction { .. } => "mutation SubmitTransaction($transaction: String!) { submitTransaction(transaction: $transaction) { hash } }",
        }
    }

    fn variables(&self) -> GraphqlVariables {
        match self {
            Self::Tip | Self::NetworkMagic => GraphqlVariables::default(),
            Self::Block { number } => GraphqlVariables {
                block_number: Some(*number),
                ..Default::default()
            },
            Self::AddressTransactions { address, limit } => GraphqlVariables {
                address: Some(address.clone()),
                limit: Some(*limit),
                ..Default::default()
            },
            Self::Balance { address } | Self::Utxos { address } => GraphqlVariables {
                address: Some(address.clone()),
                ..Default::default()
            },
            Self::SubmitTransaction { transaction } => GraphqlVariables {
                transaction: Some(transaction.clone()),
                ..Default::default()
            },
        }
    }
}
