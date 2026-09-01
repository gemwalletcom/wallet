use primitives::GEM_ANDROID_PACKAGE_ID;

#[derive(uniffi::Enum, Clone)]
pub enum PublicUrl {
    Website,
    Assets,
    PrivacyPolicy,
    TermsOfService,
    Support,
    CodebaseIos,
    CodebaseAndroid,
    AppStore,
    PlayStore,
    APK,
}

pub const ASSETS_URL: &str = "https://assets.gemwallet.com";

#[uniffi::export]
impl PublicUrl {
    pub fn url(&self) -> String {
        match self {
            Self::Website => "https://gemwallet.com".to_string(),
            Self::Assets => ASSETS_URL.to_string(),
            Self::PrivacyPolicy => "https://gemwallet.com/privacy".to_string(),
            Self::TermsOfService => "https://gemwallet.com/terms".to_string(),
            Self::Support => "https://gemwallet.com/support".to_string(),
            Self::CodebaseIos => "https://github.com/gemwalletcom/gem-ios/".to_string(),
            Self::CodebaseAndroid => "https://github.com/gemwalletcom/gem-android/".to_string(),
            Self::AppStore => "https://apps.apple.com/app/apple-store/id6448712670".to_string(),
            Self::PlayStore => format!("https://play.google.com/store/apps/details?id={GEM_ANDROID_PACKAGE_ID}"),
            Self::APK => "https://apk.gemwallet.com/gem_wallet_latest.apk".to_string(),
        }
    }
}
