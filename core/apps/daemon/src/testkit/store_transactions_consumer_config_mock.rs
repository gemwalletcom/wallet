use primitives::{DAY, HOUR, MINUTE};

use crate::consumers::store::StoreTransactionsConsumerConfig;

impl StoreTransactionsConsumerConfig {
    pub fn mock() -> Self {
        Self {
            swap_outdated_timeout: HOUR * 2,
            outdated_block_count: 12,
            outdated_min_timeout: MINUTE * 15,
            max_asset_transfer_count: 10,
            min_amount_usd: 0.01,
            primary_price_max_age: DAY,
        }
    }
}
