use std::str::FromStr;

use primitives::{AssetLink, LinkType};

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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSocialLinks {
    pub links: Vec<AssetLink>,
}

#[uniffi::export]
impl GemSocialLinks {
    pub fn sorted(&self) -> Vec<AssetLink> {
        let mut links = self.links.clone();
        links.sort_by_key(|link| std::cmp::Reverse(LinkType::from_str(&link.name).map(link_type_order).unwrap_or(0)));
        links
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorted_links_put_the_website_first_and_unknown_links_last() {
        let links = vec![
            AssetLink::new("https://t.me/gem", LinkType::Telegram),
            AssetLink {
                name: "unknown".to_string(),
                url: "https://unknown".to_string(),
            },
            AssetLink::new("https://x.com/gem", LinkType::X),
            AssetLink::new("https://gem.com", LinkType::Website),
        ];

        let names: Vec<String> = GemSocialLinks { links }.sorted().into_iter().map(|link| link.name).collect();

        assert_eq!(names, vec!["website", "x", "telegram", "unknown"]);
    }
}
