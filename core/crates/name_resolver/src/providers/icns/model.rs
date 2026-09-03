use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub bech32_address: String,
}
