use primitives::name::NameRecord;
use primitives::{AddressName, AddressType, Chain};

use super::rules;
use crate::services::recipient::{GemRecipientValidation, rules as recipient_rules};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNameRecordState {
    None,
    Loading { name: String, chain: Chain },
    Error,
    Complete { record: NameRecord },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNameIndicator {
    Loading,
    Error,
    Success,
}

#[uniffi::export]
impl GemNameRecordState {
    pub fn record(&self) -> Option<NameRecord> {
        self.record_ref().cloned()
    }

    pub fn indicator(&self) -> Option<GemNameIndicator> {
        match self {
            Self::None => None,
            Self::Loading { .. } => Some(GemNameIndicator::Loading),
            Self::Error => Some(GemNameIndicator::Error),
            Self::Complete { .. } => Some(GemNameIndicator::Success),
        }
    }

    pub fn validate_recipient(&self, chain: Chain, input: String) -> GemRecipientValidation {
        recipient_rules::validation(chain, &input, self)
    }

    pub fn name_input_step(&self, name: String, chain: Option<Chain>) -> GemNameInputStep {
        rules::name_input_step(self, &name, chain)
    }

    pub fn resolved_state(&self, name: String, chain: Chain, resolved: GemNameRecordState) -> GemNameRecordState {
        rules::resolved_state(self, &name, chain, resolved)
    }
}

impl GemNameRecordState {
    pub fn requested(&self) -> Option<(String, Chain)> {
        match self {
            Self::Loading { name, chain } => Some((name.clone(), *chain)),
            Self::Complete { record } => Some((record.name.clone(), record.chain)),
            Self::None | Self::Error => None,
        }
    }

    pub fn record_ref(&self) -> Option<&NameRecord> {
        match self {
            Self::Complete { record } => Some(record),
            Self::None | Self::Loading { .. } | Self::Error => None,
        }
    }

    pub fn can_validate_recipient(&self) -> bool {
        match self {
            Self::None | Self::Complete { .. } => true,
            Self::Loading { .. } | Self::Error => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNameInputStep {
    Unchanged,
    Reset,
    Resolve { name: String, debounce_milliseconds: u64 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddressNameUpdate {
    pub name: AddressName,
    pub replaces_types: Vec<AddressType>,
}
