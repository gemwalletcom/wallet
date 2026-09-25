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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PriceAlertNotification {
    pub wallet_id: WalletId,
    pub alert: PriceAlert,
    pub chain: Chain,
    pub state: ConnectionState,
    pub count: i32,
    pub amount: f64,
    pub votes: u32,
    pub value: BigInt,
    pub read: bool,
    pub created_at: DateTime<Utc>,
    pub accounts: Vec<Account>,
    pub note: Option<NFTImages>,
    pub path: String,
}
