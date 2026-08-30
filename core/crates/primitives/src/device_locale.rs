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

const CHINESE_SIMPLIFIED_SCRIPT: &str = "Hans";
const CHINESE_TRADITIONAL_SCRIPT: &str = "Hant";
const CHINESE_TRADITIONAL_REGIONS: [&str; 3] = ["TW", "HK", "MO"];

impl DeviceLocale {
    pub fn from_locale_identifier(identifier: &str) -> Self {
        let subtags: Vec<&str> = identifier.split(['-', '_']).filter(|subtag| !subtag.is_empty()).collect();
        let Some((language, remainder)) = subtags.split_first() else {
            return Self::EN;
        };
        let language = language.to_lowercase();
        let tag = match language.as_str() {
            "zh" => format!("zh-{}", Self::chinese_script(remainder)),
            _ => language,
        };
        Self::from_client(&tag).unwrap_or(Self::EN)
    }

    fn chinese_script(subtags: &[&str]) -> &'static str {
        if let Some(script) = subtags
            .iter()
            .find(|subtag| subtag.len() == 4 && subtag.chars().all(|character| character.is_ascii_alphabetic()))
        {
            return match script.eq_ignore_ascii_case(CHINESE_TRADITIONAL_SCRIPT) {
                true => CHINESE_TRADITIONAL_SCRIPT,
                false => CHINESE_SIMPLIFIED_SCRIPT,
            };
        }
        let is_traditional_region = subtags
            .iter()
            .any(|subtag| CHINESE_TRADITIONAL_REGIONS.iter().any(|region| region.eq_ignore_ascii_case(subtag)));
        match is_traditional_region {
            true => CHINESE_TRADITIONAL_SCRIPT,
            false => CHINESE_SIMPLIFIED_SCRIPT,
        }
    }

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

    #[test]
    fn test_device_locale_from_locale_identifier() {
        for (identifier, expected) in [
            ("en", DeviceLocale::EN),
            ("en-US", DeviceLocale::EN),
            ("en_GB", DeviceLocale::EN),
            ("en_CH", DeviceLocale::EN),
            ("EN-us", DeviceLocale::EN),
            ("de_CH", DeviceLocale::DE),
            ("fr-CA", DeviceLocale::FR),
            ("ru_UA", DeviceLocale::RU),
            ("ar_SA", DeviceLocale::AR),
            ("in-ID", DeviceLocale::ID),
            ("iw-IL", DeviceLocale::HE),
            ("tl-PH", DeviceLocale::FIL),
            ("fil-PH", DeviceLocale::FIL),
            ("pt", DeviceLocale::PtBR),
            ("pt-BR", DeviceLocale::PtBR),
            ("pt-PT", DeviceLocale::PtBR),
            ("pt_MZ", DeviceLocale::PtBR),
            ("zh", DeviceLocale::ZhHans),
            ("zh-Hans", DeviceLocale::ZhHans),
            ("zh-CN", DeviceLocale::ZhHans),
            ("zh_SG", DeviceLocale::ZhHans),
            ("zh-Hans-TW", DeviceLocale::ZhHans),
            ("zh-Hant", DeviceLocale::ZhHant),
            ("zh-TW", DeviceLocale::ZhHant),
            ("zh_HK", DeviceLocale::ZhHant),
            ("zh-MO", DeviceLocale::ZhHant),
            ("zh-Hant-CN", DeviceLocale::ZhHant),
            ("af-ZA", DeviceLocale::EN),
            ("sv-SE", DeviceLocale::EN),
            ("zz-ZZ", DeviceLocale::EN),
            ("", DeviceLocale::EN),
            ("-", DeviceLocale::EN),
        ] {
            assert_eq!(DeviceLocale::from_locale_identifier(identifier), expected, "identifier: {identifier}");
        }
    }
}
