use std::str::FromStr;

use primitives::{AssetLink, LinkType};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSocialLink {
    pub link_type: LinkType,
    pub url: String,
    pub host: Option<String>,
}

#[uniffi::export]
pub fn community_links() -> Vec<GemSocialLink> {
    let links = [
        (LinkType::X, "https://x.com/GemWallet"),
        (LinkType::Discord, "https://discord.gg/aWkq5sj7SY"),
        (LinkType::Telegram, "https://t.me/gemwallet"),
        (LinkType::GitHub, "https://github.com/gemwalletcom"),
        (LinkType::YouTube, "https://www.youtube.com/@gemwallet"),
    ]
    .into_iter()
    .map(|(link_type, url)| AssetLink::new(url, link_type))
    .collect();
    social_links(links)
}

#[uniffi::export]
pub fn social_links(links: Vec<AssetLink>) -> Vec<GemSocialLink> {
    let mut rows: Vec<GemSocialLink> = links.iter().filter_map(social_link).collect();
    rows.sort_by_key(|row| std::cmp::Reverse(link_type_order(row.link_type)));
    rows
}

fn social_link(link: &AssetLink) -> Option<GemSocialLink> {
    let link_type = link_type(&link.name)?;
    let host = (link_type == LinkType::Website).then(|| host(&link.url)).flatten();
    Some(GemSocialLink {
        link_type,
        url: link.url.clone(),
        host,
    })
}

fn link_type(name: &str) -> Option<LinkType> {
    match name {
        "twitter" => Some(LinkType::X),
        name => LinkType::from_str(name).ok(),
    }
}

fn host(url: &str) -> Option<String> {
    url::Url::parse(url).ok()?.host_str().map(|host| host.trim_start_matches("www.").to_string())
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
    fn test_community_links_come_in_display_order() {
        let types: Vec<LinkType> = community_links().into_iter().map(|link| link.link_type).collect();
        assert_eq!(types, [LinkType::X, LinkType::Telegram, LinkType::YouTube, LinkType::GitHub, LinkType::Discord]);
    }

    #[test]
    fn test_social_links_put_the_website_first_drop_unknown_names_and_carry_the_host() {
        let links = vec![
            AssetLink::new("https://t.me/gem", LinkType::Telegram),
            AssetLink {
                name: "unknown".to_string(),
                url: "https://unknown".to_string(),
            },
            AssetLink {
                name: "twitter".to_string(),
                url: "https://x.com/gem".to_string(),
            },
            AssetLink::new("https://www.gem.com/about", LinkType::Website),
        ];

        let rows = social_links(links);
        let types: Vec<LinkType> = rows.iter().map(|row| row.link_type).collect();

        assert_eq!(types, vec![LinkType::Website, LinkType::X, LinkType::Telegram]);
        assert_eq!(rows[0].host.as_deref(), Some("gem.com"));
        assert_eq!(rows[1].host, None);
    }
}
