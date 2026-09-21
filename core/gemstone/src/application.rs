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

    pub fn connection_row(&self, metadata: ApplicationMetadata) -> GemConnectionRow {
        let name = metadata.short_name();
        GemConnectionRow {
            initial: name.chars().next().map(|first| first.to_uppercase().to_string()),
            title: name,
            host: Some(url_host(&metadata.url)).filter(|host| !host.is_empty()),
            icon_url: self.icon_url(metadata),
        }
    }

    pub fn host(&self, metadata: ApplicationMetadata) -> String {
        url_host(&metadata.url)
    }

    pub fn icon_url(&self, metadata: ApplicationMetadata) -> Option<String> {
        let website = public_url(&metadata.url)?;
        let icon = public_url(&metadata.icon).filter(|icon| icon.fragment().is_none() && icon.as_str().len() <= ICON_URL_MAX_LENGTH);
        let source = icon.map_or_else(|| website.origin().ascii_serialization(), |icon| icon.to_string());
        let query = form_urlencoded::Serializer::new(String::new()).append_pair("url", &source).append_pair("size", APPLICATION_ICON_SIZE).finish();
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
    (url.scheme() == "https" && url.username().is_empty() && url.password().is_none() && url.port().is_none() && host.contains('.') && !blocked.iter().any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_connection_without_a_host_has_no_subtitle_and_takes_its_initial_from_the_name() {
        let service = GemApplicationMetadataService::new();

        let hosted = service.connection_row(ApplicationMetadata {
            name: "Uniswap".to_string(),
            url: "https://app.uniswap.org".to_string(),
            icon: String::new(),
            ..ApplicationMetadata::mock()
        });
        assert_eq!(hosted.host.as_deref(), Some("app.uniswap.org"));
        assert_eq!(hosted.initial.as_deref(), Some("U"));

        assert_eq!(hosted.icon_url.as_deref(), Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Fapp.uniswap.org&size=256"));

        let unhosted = ApplicationMetadata {
            name: "Uniswap".to_string(),
            url: String::new(),
            ..ApplicationMetadata::mock()
        };
        assert_eq!(service.connection_row(unhosted.clone()).host, None);
        assert_eq!(service.connection_row(ApplicationMetadata { name: String::new(), ..unhosted.clone() }).initial, None);
        assert_eq!(service.connection_row(unhosted).icon_url, None);
    }

    #[test]
    fn test_host_drops_the_scheme_path_and_www_prefix() {
        let service = GemApplicationMetadataService::new();

        assert_eq!(
            service.host(ApplicationMetadata {
                url: "https://www.venice.ai/path?query=1#fragment".into(),
                ..ApplicationMetadata::mock()
            }),
            "venice.ai"
        );
        assert_eq!(
            service.host(ApplicationMetadata {
                url: "http://www.venice.ai".into(),
                ..ApplicationMetadata::mock()
            }),
            "venice.ai"
        );
        assert_eq!(
            service.host(ApplicationMetadata {
                url: "www.venice.ai/path".into(),
                ..ApplicationMetadata::mock()
            }),
            "venice.ai"
        );
        assert_eq!(
            service.host(ApplicationMetadata {
                url: "app.uniswap.org".into(),
                ..ApplicationMetadata::mock()
            }),
            "app.uniswap.org"
        );
        assert_eq!(
            service.host(ApplicationMetadata {
                url: " ".into(),
                ..ApplicationMetadata::mock()
            }),
            ""
        );
    }

    #[test]
    fn test_icon_url_requests_the_application_icon() {
        assert_eq!(
            GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                url: "https://login.xyz/some/page?query=value#section".into(),
                icon: "https://login.xyz/favicon.png".into(),
                ..ApplicationMetadata::mock()
            }),
            Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Flogin.xyz%2Ffavicon.png&size=256".into())
        );
        assert_eq!(
            GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                url: "https://tronscan.org".into(),
                icon: "https://cdn.example.com/logo.svg?v=2".into(),
                ..ApplicationMetadata::mock()
            }),
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
                GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                    url: "https://tronscan.org/some/page?query=value#section".into(),
                    icon: icon.into(),
                    ..ApplicationMetadata::mock()
                }),
                Some("https://assets.gemwallet.com/proxy/icon?url=https%3A%2F%2Ftronscan.org&size=256".into()),
                "icon: {icon}"
            );
        }
    }

    #[test]
    fn test_icon_url_omits_missing_or_unsafe_websites() {
        for url in ["", " ", "http://app.example.com", "data:text/html,hello", "https://user:password@app.example.com", "https://app.example.com:8443", "https://["] {
            assert_eq!(
                GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                    url: url.into(),
                    icon: "https://cdn.example.com/icon.png".into(),
                    ..ApplicationMetadata::mock()
                }),
                None,
                "url: {url}"
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemConnectionRow {
    pub title: String,
    pub host: Option<String>,
    pub initial: Option<String>,
    pub icon_url: Option<String>,
}
