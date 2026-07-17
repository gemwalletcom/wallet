use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

pub const MIME_TYPE_PNG: &str = "image/png";
const MIME_TYPE_JPEG: &str = "image/jpeg";
const MIME_TYPE_SVG: &str = "image/svg+xml";
const MIME_TYPE_GIF: &str = "image/gif";
const MIME_TYPE_WEBP: &str = "image/webp";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ImageType {
    Gif,
    Jpeg,
    Jpg,
    Png,
    Svg,
    Webp,
}

impl ImageType {
    pub fn from_label(value: &str) -> Option<Self> {
        Self::from_str(value).ok()
    }

    pub fn from_extension(file_name: &str) -> Option<Self> {
        Self::from_label(file_name.rsplit_once('.')?.1)
    }

    pub fn from_magic_bytes(data: &[u8]) -> Option<Self> {
        Self::iter().find(|image_type| image_type.matches_magic_bytes(data))
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Gif => MIME_TYPE_GIF,
            Self::Jpeg | Self::Jpg => MIME_TYPE_JPEG,
            Self::Png => MIME_TYPE_PNG,
            Self::Svg => MIME_TYPE_SVG,
            Self::Webp => MIME_TYPE_WEBP,
        }
    }

    pub fn extension(self) -> String {
        self.as_ref().to_string()
    }

    fn matches_magic_bytes(self, data: &[u8]) -> bool {
        match self {
            Self::Gif => data.starts_with(b"GIF8"),
            Self::Jpeg | Self::Jpg => data.starts_with(&[0xFF, 0xD8, 0xFF]),
            Self::Png => data.starts_with(b"\x89PNG\r\n\x1A\n"),
            Self::Svg => data.starts_with(b"<svg"),
            Self::Webp => data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_labels() {
        assert_eq!(ImageType::from_label("jpg"), Some(ImageType::Jpg));
        assert_eq!(ImageType::from_label("jpeg"), Some(ImageType::Jpeg));
        assert_eq!(ImageType::from_label("png"), Some(ImageType::Png));
        assert_eq!(ImageType::from_label("gif"), Some(ImageType::Gif));
        assert_eq!(ImageType::from_label("webp"), Some(ImageType::Webp));
        assert_eq!(ImageType::from_label("html"), None);
    }

    #[test]
    fn detects_image_signatures() {
        assert_eq!(ImageType::from_magic_bytes(&[0xFF, 0xD8, 0xFF]), Some(ImageType::Jpeg));
        assert_eq!(ImageType::from_magic_bytes(b"\x89PNG\r\n\x1A\nrest"), Some(ImageType::Png));
        assert_eq!(ImageType::from_magic_bytes(b"GIF89a"), Some(ImageType::Gif));
        assert_eq!(ImageType::from_magic_bytes(b"RIFF\x01\x02\x03\x04WEBPVP8 "), Some(ImageType::Webp));
        assert_eq!(ImageType::from_magic_bytes(b"RIFF\x01\x02\x03\x04WAVEfmt "), None);
        assert_eq!(ImageType::from_magic_bytes(b"<html></html>"), None);
    }
}
