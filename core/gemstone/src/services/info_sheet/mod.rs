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
impl GemInfoSheet {
    /// The button the sheet shows: a docs link always opens, any other action only when the screen handles it.
    pub fn button(&self, handles_actions: bool) -> Option<GemInfoAction> {
        self.action.clone().filter(|action| matches!(action, GemInfoAction::LearnMore { .. }) || handles_actions)
    }
}

#[uniffi::export]
impl GemConfirmErrorInfo {
    pub fn sheet(&self, platform: Platform) -> GemInfoSheet {
        rules::confirm_error_sheet(self, platform)
    }
}
