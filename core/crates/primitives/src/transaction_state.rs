use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::swap::SwapStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsRefStr, EnumIter, EnumString, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
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
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            Self::Pending | Self::InTransit => SwapStatus::Pending,
            Self::Confirmed => SwapStatus::Completed,
            Self::Failed | Self::Reverted => SwapStatus::Failed,
            Self::Refunded => SwapStatus::Refunded,
        }
    }

    pub fn pending() -> Vec<Self> {
        Self::iter().filter(|state| !state.is_completed()).collect()
    }

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

    #[test]
    fn test_pending_states_are_the_ones_not_completed() {
        assert_eq!(TransactionState::pending(), vec![TransactionState::Pending, TransactionState::InTransit]);
    }
}
