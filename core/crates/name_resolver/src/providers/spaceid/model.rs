use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResolveRecord {
    pub code: i32,
    pub address: String,
}
