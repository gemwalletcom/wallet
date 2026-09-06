#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub chain: Chain,
    pub address: String,
    pub derivation_path: String,
    pub extended_public_key: Option<String>,
}
