use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ZerionPositionsResponse {
    pub data: Vec<ZerionPosition>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionPosition {
    pub id: String,
    pub attributes: ZerionPositionAttributes,
    pub relationships: ZerionPositionRelationships,
}

#[derive(Debug, Deserialize)]
pub struct ZerionPositionAttributes {
    pub name: String,
    pub quantity: ZerionQuantity,
    pub protocol: Option<String>,
    pub protocol_module: Option<String>,
    pub position_type: Option<String>,
    pub fungible_info: ZerionFungibleInfo,
    pub application_metadata: Option<ZerionApplicationMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionQuantity {
    pub int: String,
}

#[derive(Debug, Deserialize)]
pub struct ZerionFungibleInfo {
    pub implementations: Vec<ZerionFungibleImplementation>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionFungibleImplementation {
    pub chain_id: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionApplicationMetadata {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionPositionRelationships {
    pub dapp: Option<ZerionRelationship>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionRelationship {
    pub data: Option<ZerionRelationshipData>,
}

#[derive(Debug, Deserialize)]
pub struct ZerionRelationshipData {
    pub id: String,
}
