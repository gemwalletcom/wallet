use chrono::DateTime;
use primitives::Chain;

use crate::models::ParserStateRow;
use crate::sql_types::ChainRow;

impl ParserStateRow {
    pub fn mock() -> Self {
        Self {
            chain: ChainRow::from(Chain::Ethereum),
            current_block: 0,
            latest_block: 0,
            await_blocks: 1,
            timeout_between_blocks: 0,
            timeout_latest_block: 0,
            parallel_blocks: 1,
            is_enabled: true,
            updated_at: DateTime::UNIX_EPOCH.naive_utc(),
            queue_behind_blocks: None,
            block_time: 0,
        }
    }
}
