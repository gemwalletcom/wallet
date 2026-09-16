use serde::{Deserialize, Serialize};

use crate::known_entries::deserialize_known_entries;
use crate::{AssetPrice, FiatRate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketPricePayload {
    #[serde(deserialize_with = "deserialize_known_entries")]
    pub prices: Vec<AssetPrice>,
    #[serde(deserialize_with = "deserialize_known_entries")]
    pub rates: Vec<FiatRate>,
}
