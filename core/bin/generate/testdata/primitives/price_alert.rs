#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    #[typeshare(skip)]
    #[serde(skip)]
    pub identifier: String,
    pub price: Option<f64>,
    #[serde(rename = "type")]
    pub kind: String,
    pub last_notified_at: Option<DateTime<Utc>>,
}
