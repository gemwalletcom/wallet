use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResolverRecord {
    pub address: String,
}
