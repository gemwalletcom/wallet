use crate::services::failures::StepFailure;
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAppStartStep {
    UpdateConfig,
    SetupBanners,
    SyncAssets,
    SyncDevice,
    SetupWalletAssets,
    SetupWalletBanners,
    SyncWalletConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAppStartFailure {
    pub step: GemAppStartStep,
    pub message: String,
}

impl StepFailure for GemAppStartFailure {
    type Step = GemAppStartStep;

    fn new(step: GemAppStartStep, message: String) -> Self {
        Self { step, message }
    }
}
