use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LinkType {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
    Facebook,
    Website,
    Coingecko,
    OpenSea,
    Instagram,
    MagicEden,
    CoinMarketCap,
    TikTok,
}

impl LinkType {
    pub fn name(&self) -> String {
        self.as_ref().to_string()
    }
}
