pub mod model;
pub mod rules;

pub use model::{GemInfoAction, GemInfoAmount, GemInfoDescription, GemInfoImage, GemInfoSheet, GemInfoTitle};

use primitives::Platform;

use crate::models::list::GemInfoTopic;
use crate::services::confirm::error::GemConfirmErrorInfo;

#[uniffi::export]
impl GemInfoTopic {
    pub fn sheet(&self, platform: Platform) -> GemInfoSheet {
        rules::sheet(self, platform)
    }
}

#[uniffi::export]
impl GemConfirmErrorInfo {
    pub fn sheet(&self, platform: Platform) -> GemInfoSheet {
        rules::confirm_error_sheet(self, platform)
    }
}
