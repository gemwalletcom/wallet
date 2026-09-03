use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Deserialize, Serialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Blocks {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Block {
    pub number: i64,
    pub hash: String,
    #[serde(rename = "forgedAt")]
    pub forged_at: String,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub hash: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub fee: BigUint,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressTransactions {
    pub transactions: Vec<AddressTransaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressTransaction {
    #[serde(flatten)]
    pub transaction: Transaction,
    #[serde(rename = "includedAt")]
    pub included_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<&'static str>,
    pub variables: GraphqlVariables,
    pub query: &'static str,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlVariables {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
}
