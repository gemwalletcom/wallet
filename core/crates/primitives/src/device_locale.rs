use serde::{Deserialize, Deserializer, Serialize, de};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Clone, Copy, Debug, Serialize, AsRefStr, EnumString, PartialEq, Eq)]
#[typeshare(swift = "CaseIterable, Equatable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DeviceLocale {
    AR,
    BN,
    CS,
    DA,
    DE,
    EN,
    ES,
    FA,
    FIL,
    FR,
    HA,
    HE,
    HI,
    ID,
    IT,
    JA,
    KO,
    MS,
    NL,
    PL,
    #[serde(rename = "pt-BR")]
    #[strum(serialize = "pt-BR")]
    PtBR,
    RO,
    RU,
    SW,
    TH,
    TR,
    UK,
    UR,
    VI,
    #[serde(rename = "zh-Hans")]
    #[strum(serialize = "zh-Hans")]
    ZhHans,
    #[serde(rename = "zh-Hant")]
    #[strum(serialize = "zh-Hant")]
    ZhHant,
}

impl DeviceLocale {
    pub fn from_client(locale: &str) -> Result<Self, String> {
        if let Ok(locale) = locale.parse() {
            return Ok(locale);
        }

        match locale {
            // TODO: Remove legacy locale compatibility after clients send DeviceLocale.
            "in" => Ok(Self::ID),
            "iw" => Ok(Self::HE),
            "pt" => Ok(Self::PtBR),
            "tl" => Ok(Self::FIL),
            "zh" => Ok(Self::ZhHans),
            "af" | "am" | "az" | "be" | "bg" | "bs" | "ca" | "ckb" | "el" | "et" | "fi" | "gl" | "gu" | "hr" | "hu" | "hy" | "is" | "ka" | "kk" | "km" | "kn" | "ky" | "lo"
            | "lt" | "lv" | "mfe" | "mg" | "mk" | "mn" | "mr" | "my" | "nb" | "ne" | "om" | "or" | "pa" | "sk" | "sl" | "so" | "sq" | "sr" | "sv" | "ta" | "te" | "tk" | "uz" => {
                Ok(Self::EN)
            }
            _ => Err(format!("invalid device locale: {locale}")),
        }
    }
}

impl<'de> Deserialize<'de> for DeviceLocale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let locale = String::deserialize(deserializer)?;
        Self::from_client(&locale).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceLocale;

    #[test]
    fn test_device_locale_deserialization() {
        for (locale, expected) in [
            ("ar", DeviceLocale::AR),
            ("fil", DeviceLocale::FIL),
            ("pt-BR", DeviceLocale::PtBR),
            ("zh-Hant", DeviceLocale::ZhHant),
            ("in", DeviceLocale::ID),
            ("iw", DeviceLocale::HE),
            ("pt", DeviceLocale::PtBR),
            ("tl", DeviceLocale::FIL),
            ("zh", DeviceLocale::ZhHans),
            ("hu", DeviceLocale::EN),
            ("mfe", DeviceLocale::EN),
            ("sv", DeviceLocale::EN),
            ("uz", DeviceLocale::EN),
        ] {
            assert_eq!(serde_json::from_str::<DeviceLocale>(&format!("\"{locale}\"")).unwrap(), expected);
        }

        for locale in ["", "zz", "aus", "null", "EN", "en-US", "en_US", "pt-PT", "zh-CN"] {
            assert!(serde_json::from_str::<DeviceLocale>(&format!("\"{locale}\"")).is_err());
        }
    }
}
