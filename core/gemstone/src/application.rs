use primitives::ApplicationMetadata;
use url::{Url, form_urlencoded};

use crate::config::public::ASSETS_URL;

const APPLICATION_ICON_SIZE: &str = "256";

#[derive(Default, uniffi::Object)]
pub struct GemApplicationMetadataService {}

#[uniffi::export]
impl GemApplicationMetadataService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn short_name(&self, metadata: ApplicationMetadata) -> String {
        metadata.short_name()
    }

    pub fn host(&self, metadata: ApplicationMetadata) -> String {
        url_host(&metadata.url)
    }

    pub fn icon_url(&self, metadata: ApplicationMetadata) -> Option<String> {
        let source = Url::parse(&metadata.url).ok()?;
        if source.scheme() != "https" || !source.username().is_empty() || source.password().is_some() || source.port().is_some() {
            return None;
        }
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("url", &source.origin().ascii_serialization())
            .append_pair("size", APPLICATION_ICON_SIZE)
            .finish();
        Some(format!("{ASSETS_URL}/proxy/icon?{query}"))
    }
}

pub fn url_host(url: &str) -> String {
    let url = url.trim();
    let host = match Url::parse(url) {
        Ok(parsed) => parsed.host_str().map(str::to_string).unwrap_or_else(|| url.to_string()),
        Err(_) => url.rsplit("://").next().unwrap_or(url).split(['/', '?', '#']).next().unwrap_or_default().to_string(),
    };
    host.trim_start_matches("www.").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_drops_the_scheme_path_and_www_prefix() {
        let host = |url: &str| {
            GemApplicationMetadataService::new().host(ApplicationMetadata {
                url: url.into(),
                ..ApplicationMetadata::mock()
            })
        };

        assert_eq!(host("https://www.venice.ai/path?query=1#fragment"), "venice.ai");
        assert_eq!(host("http://www.venice.ai"), "venice.ai");
        assert_eq!(host("www.venice.ai/path"), "venice.ai");
        assert_eq!(host("app.uniswap.org"), "app.uniswap.org");
        assert_eq!(host(" "), "");
    }

    #[test]
    fn test_icon_url_uses_website_origin() {
        for icon in ["", "https://tronscan.org/static/media/logo.png", "https://cdn.example.com/logo.svg", "../icon.png"] {
            assert_eq!(
                GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                    url: "https://tronscan.org/some/page?query=value#section".into(),
                    icon: icon.into(),
                    ..ApplicationMetadata::mock()
                }),
                Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Ftronscan.org&size=256".into())
            );
        }
    }

    #[test]
    fn test_icon_url_omits_missing_or_unsafe_websites() {
        for url in [
            "",
            " ",
            "http://app.example.com",
            "data:text/html,hello",
            "https://user:password@app.example.com",
            "https://app.example.com:8443",
            "https://[",
        ] {
            assert_eq!(
                GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                    url: url.into(),
                    icon: "https://cdn.example.com/icon.png".into(),
                    ..ApplicationMetadata::mock()
                }),
                None
            );
        }
    }
}
