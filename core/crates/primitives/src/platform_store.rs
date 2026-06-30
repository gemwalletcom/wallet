use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum PlatformStore {
    AppStore,
    GooglePlay,
    Fdroid,
    Huawei,
    SolanaStore,
    SamsungStore,
    ApkUniversal,
    Emerald,
    Local,
}

impl PlatformStore {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::AppStore => "App Store",
            Self::GooglePlay => "Google Play",
            Self::Fdroid => "F-Droid",
            Self::Huawei => "Huawei",
            Self::SolanaStore => "Solana Store",
            Self::SamsungStore => "Samsung Store",
            Self::ApkUniversal => "APK Universal",
            Self::Emerald => "Emerald",
            Self::Local => "Local",
        }
    }
}
