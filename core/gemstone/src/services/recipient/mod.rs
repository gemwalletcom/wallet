pub mod model;
pub mod rules;

use primitives::Chain;
use primitives::name::NameRecord;

use crate::services::name::rules::is_name_supported;
use crate::services::transfer::GemRecipient;

pub use model::{GemRecipientError, GemRecipientValidation};

#[derive(Default, uniffi::Object)]
pub struct GemRecipientService;

#[uniffi::export]
impl GemRecipientService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn is_name_supported(&self, input: String) -> bool {
        is_name_supported(&input)
    }

    pub fn validate(&self, chain: Chain, input: String, name_record: Option<NameRecord>) -> GemRecipientValidation {
        rules::validation(chain, &input, name_record.as_ref())
    }

    pub fn recipient(
        &self,
        chain: Chain,
        input: String,
        name_record: Option<NameRecord>,
        memo: Option<String>,
        references: Vec<String>,
    ) -> Result<GemRecipient, GemRecipientError> {
        rules::recipient(chain, &input, name_record.as_ref(), memo, references)
    }
}
