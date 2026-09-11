use primitives::name::NameRecord;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNameRecordState {
    None,
    Loading { name: String },
    Error,
    Complete { record: NameRecord },
}

#[uniffi::export]
impl GemNameRecordState {
    pub fn record(&self) -> Option<NameRecord> {
        self.record_ref().cloned()
    }

    pub fn requested_name(&self) -> Option<String> {
        match self {
            Self::Loading { name } => Some(name.clone()),
            Self::Complete { record } => Some(record.name.clone()),
            Self::None | Self::Error => None,
        }
    }
}

impl GemNameRecordState {
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
