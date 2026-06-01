use crate::{TransactionStateRequest, UInt64};
use chrono::{DateTime, Utc};

impl TransactionStateRequest {
    pub fn mock_with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            sender_address: String::new(),
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            block_number: 0,
        }
    }

    pub fn with_block_number(mut self, block_number: UInt64) -> Self {
        self.block_number = block_number;
        self
    }
}
