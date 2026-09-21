use primitives::name::NameRecord;
use primitives::{AddressName, AddressType, Chain};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNameRecordState {
    None,
    Loading { name: String, chain: Chain },
    Error,
    Complete { record: NameRecord },
}

#[uniffi::export]
impl GemNameRecordState {
    pub fn record(&self) -> Option<NameRecord> {
        self.record_ref().cloned()
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
