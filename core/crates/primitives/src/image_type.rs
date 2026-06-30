use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

pub const MIME_TYPE_PNG: &str = "image/png";
const MIME_TYPE_JPEG: &str = "image/jpeg";
const MIME_TYPE_SVG: &str = "image/svg+xml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ImageType {
    Jpeg,
    Jpg,
    Png,
    Svg,
}

impl ImageType {
    pub fn from_label(value: &str) -> Option<Self> {
        Self::from_str(value).ok()
    }

    pub fn from_extension(file_name: &str) -> Option<Self> {
        Self::from_label(file_name.rsplit_once('.')?.1)
    }

    pub fn from_magic_bytes(data: &[u8]) -> Option<Self> {
        Self::iter().find(|image_type| data.starts_with(image_type.magic_bytes()))
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Jpeg | Self::Jpg => MIME_TYPE_JPEG,
            Self::Png => MIME_TYPE_PNG,
            Self::Svg => MIME_TYPE_SVG,
        }
    }

    pub fn extension(self) -> String {
        self.as_ref().to_string()
    }

    fn magic_bytes(self) -> &'static [u8] {
        match self {
            Self::Jpeg | Self::Jpg => &[0xFF, 0xD8, 0xFF],
            Self::Png => b"\x89PNG\r\n\x1A\n",
            Self::Svg => b"<svg",
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
        assert_eq!(ImageType::from_label("html"), None);
    }

    #[test]
    fn detects_image_signatures() {
        assert_eq!(ImageType::from_magic_bytes(&[0xFF, 0xD8, 0xFF]), Some(ImageType::Jpeg));
        assert_eq!(ImageType::from_magic_bytes(b"\x89PNG\r\n\x1A\nrest"), Some(ImageType::Png));
        assert_eq!(ImageType::from_magic_bytes(b"<html></html>"), None);
    }
}
