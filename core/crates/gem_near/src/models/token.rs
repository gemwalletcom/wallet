use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FungibleTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Deserialize)]
pub struct StorageBalanceBounds {
    pub min: String,
}
