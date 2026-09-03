use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResolveRecord {
    pub owner: String,
}
