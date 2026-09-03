use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub name: RecordName,
    pub data: RecordData,
}

#[derive(Debug, Deserialize)]
pub struct RecordName {
    pub resolved: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordData {
    pub chain_addresses: HashMap<String, String>,
}
