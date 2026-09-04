pub mod model;
pub mod rules;
pub mod service;

pub use model::{GemRecipientError, GemRecipientValidation};
pub use service::GemRecipientService;

use crate::services::transfer::GemRecipient;

#[uniffi::export]
pub fn recipient_id(recipient: GemRecipient) -> String {
    rules::recipient_id(&recipient)
}
