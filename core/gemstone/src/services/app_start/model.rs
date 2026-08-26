#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAppStartStep {
    UpdateConfig,
    SetupBanners,
    SyncAssets,
    SetupWalletAssets,
    SetupWalletBanners,
    SyncWalletConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAppStartFailure {
    pub step: GemAppStartStep,
    pub message: String,
}
