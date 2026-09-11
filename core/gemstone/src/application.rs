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

    pub fn icon_url(&self, metadata: ApplicationMetadata) -> Option<String> {
        if metadata.icon.trim().is_empty() {
            return None;
        }
        let source = Url::parse(&metadata.icon).or_else(|_| Url::parse(&metadata.url)?.join(&metadata.icon)).ok()?;
        if source.scheme() != "https" || !source.username().is_empty() || source.password().is_some() {
            return None;
        }
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("url", source.as_str())
            .append_pair("size", APPLICATION_ICON_SIZE)
            .finish();
        Some(format!("{ASSETS_URL}/proxy/image?{query}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_url_encodes_absolute_and_relative_sources() {
        let service = GemApplicationMetadataService::new();
        for icon in ["https://app.example.com/icons/logo.svg?v=1&theme=dark", "../icons/logo.svg?v=1&theme=dark"] {
            let metadata = ApplicationMetadata {
                url: "https://app.example.com/swap/".into(),
                icon: icon.into(),
                ..ApplicationMetadata::mock()
            };
            assert_eq!(
                service.icon_url(metadata),
                Some("https://assets.gemwallet.com/proxy/image?url=https%3A%2F%2Fapp.example.com%2Ficons%2Flogo.svg%3Fv%3D1%26theme%3Ddark&size=256".into())
            );
        }
    }

    #[test]
    fn test_icon_url_omits_missing_or_unsafe_sources() {
        for icon in [
            "",
            " ",
            "http://app.example.com/icon.png",
            "data:image/svg+xml,<svg/>",
            "https://user:password@app.example.com/icon.png",
            "https://[",
        ] {
            assert_eq!(
                GemApplicationMetadataService::new().icon_url(ApplicationMetadata {
                    url: "https://app.example.com".into(),
                    icon: icon.into(),
                    ..ApplicationMetadata::mock()
                }),
                None
            );
        }
    }
}
