#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAccountRow {
    pub account: Account,
    pub wallet_id: WalletId,
    pub chain: Chain,
    pub state: ConnectionState,
    pub balance: GemBigInt,
    pub title: String,
    pub subtitle: Option<String>,
    pub rows: Vec<GemRow>,
    pub icon: GemIcon,
    pub badge: RowIcon,
    pub failure: Option<GemRowError>,
    pub data: Vec<u8>,
    pub labels: HashMap<String, String>,
    pub action: GemRowAction,
    pub r#type: String,
    pub protocol: String,
    pub created_at: DateTimeUtc,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemRowError {
    Missing,
}

pub fn row(id: String) -> Result<GemAccountRow, GemRowError> {
    Err(GemRowError::Missing)
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemIcon {
    Asset,
    Wallet,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemRowAction {
    Open { url: String, chain: Chain },
    Copy(String),
    None,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPayment {
    pub action: GemPaymentAction,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemPaymentAction {
    Send(String),
    Cancel,
}

pub struct GemNotExported {
    pub value: String,
}
