use serde::{Deserialize, Serialize};

use crate::{AssetPrice, FiatRate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketPricePayload {
    pub prices: Vec<AssetPrice>,
    pub rates: Vec<FiatRate>,
}
