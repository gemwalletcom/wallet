use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumIter, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum TransactionState {
    Pending,
    Confirmed,
    InTransit,
    Failed,
    Reverted,
    Refunded,
}

impl TransactionState {
    pub fn is_completed(&self) -> bool {
        match self {
            Self::Confirmed | Self::Failed | Self::Reverted | Self::Refunded => true,
            Self::Pending | Self::InTransit => false,
        }
    }

    pub fn merged_with(self, updated: Self) -> Self {
        if self == Self::Pending || updated.is_completed() { updated } else { self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_completed() {
        assert!(TransactionState::Confirmed.is_completed());
        assert!(TransactionState::Failed.is_completed());
        assert!(TransactionState::Reverted.is_completed());
        assert!(TransactionState::Refunded.is_completed());
        assert!(!TransactionState::Pending.is_completed());
        assert!(!TransactionState::InTransit.is_completed());
    }
}
