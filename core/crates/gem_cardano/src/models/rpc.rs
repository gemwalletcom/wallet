use serde::{Deserialize, Serialize};

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
    pub fee: String,
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
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub address: String,
    pub value: String,
}
