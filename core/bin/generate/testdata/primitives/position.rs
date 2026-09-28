#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub id: String,
    pub leverage: u8,
    pub entry_price: f64,
    pub take_profit: Option<TriggerOrder>,
    pub images: NFTImages,
    pub tags: Vec<String>,
    pub state: ConnectionState,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TriggerOrder {
    pub price: f64,
    pub order_id: String,
}
