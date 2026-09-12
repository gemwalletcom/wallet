use primitives::ApplicationMetadata;
use url::{Host, Url, form_urlencoded};

use crate::config::public::ASSETS_URL;

const APPLICATION_ICON_SIZE: &str = "256";
const ICON_URL_MAX_LENGTH: usize = 4096;

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
        let website = public_url(&metadata.url)?;
        let icon = public_url(&metadata.icon).filter(|icon| icon.fragment().is_none() && icon.as_str().len() <= ICON_URL_MAX_LENGTH);
        let source = icon.map_or_else(|| website.origin().ascii_serialization(), |icon| icon.to_string());
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("url", &source)
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

fn public_url(url: &str) -> Option<Url> {
    let url = Url::parse(url).ok()?;
    let host = match url.host() {
        Some(Host::Domain(host)) => host.trim_end_matches('.').to_string(),
        _ => return None,
    };
    let blocked = ["gemwallet.com", "workers.dev"];
    (url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && host.contains('.')
        && !blocked.iter().any(|domain| host == *domain || host.ends_with(&format!(".{domain}"))))
    .then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon_url(url: &str, icon: &str) -> Option<String> {
        GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
            url: url.into(),
            icon: icon.into(),
            ..ApplicationMetadata::mock()
        })
    }

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
    fn test_icon_url_requests_the_application_icon() {
        assert_eq!(
            icon_url("https://login.xyz/some/page?query=value#section", "https://login.xyz/favicon.png"),
            Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Flogin.xyz%2Ffavicon.png&size=256".into())
        );
        assert_eq!(
            icon_url("https://tronscan.org", "https://cdn.example.com/logo.svg?v=2"),
            Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Fcdn.example.com%2Flogo.svg%3Fv%3D2&size=256".into())
        );
    }

    #[test]
    fn test_icon_url_falls_back_to_the_website_origin() {
        for icon in [
            "",
            "../icon.png",
            "favicon.ico",
            "http://tronscan.org/logo.png",
            "data:image/png;base64,iVBORw0KGgo=",
            "https://tronscan.org/logo.png#fragment",
            "https://user:password@tronscan.org/logo.png",
            "https://tronscan.org:8443/logo.png",
            "https://192.168.1.1/logo.png",
            "https://localhost/logo.png",
            "https://assets.gemwallet.com/logo.png",
        ] {
            assert_eq!(
                icon_url("https://tronscan.org/some/page?query=value#section", icon),
                Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Ftronscan.org&size=256".into()),
                "icon: {icon}"
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
            assert_eq!(icon_url(url, "https://cdn.example.com/icon.png"), None, "url: {url}");
        }
    }
}
