use primitives::LinkType;

#[derive(uniffi::Enum, Clone)]
pub enum SocialUrl {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
    Facebook,
    Website,
    Coingecko,
}

#[uniffi::export]
impl SocialUrl {
    pub fn url(&self) -> Option<String> {
        match self {
            Self::X => Some("https://x.com/GemWallet".to_string()),
            Self::Discord => Some("https://discord.gg/aWkq5sj7SY".to_string()),
            Self::Telegram => Some("https://t.me/gemwallet".to_string()),
            Self::GitHub => Some("https://github.com/gemwalletcom".to_string()),
            Self::YouTube => Some("https://www.youtube.com/@gemwallet".to_string()),
            Self::Reddit | Self::Facebook | Self::Website | Self::Coingecko => None,
        }
    }
}

#[uniffi::export]
fn link_type_order(link_type: LinkType) -> i32 {
    match link_type {
        LinkType::Website => 120,
        LinkType::X => 110,
        LinkType::Coingecko => 105,
        LinkType::CoinMarketCap => 104,
        LinkType::OpenSea => 103,
        LinkType::MagicEden => 102,
        LinkType::Telegram => 90,
        LinkType::Reddit => 60,
        LinkType::Instagram => 50,
        LinkType::Facebook => 40,
        LinkType::TikTok => 35,
        LinkType::Discord => 1,
        LinkType::GitHub => 20,
        LinkType::YouTube => 30,
    }
}
