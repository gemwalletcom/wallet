use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, AsRefStr, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ConsumerService {
    Store,
    Indexer,
    Notifications,
    Rewards,
    Support,
    Fiat,
}

impl ConsumerService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum IndexerConsumer {
    FetchAssets,
    FetchLists,
    FetchPrices,
    FetchBlocks,
    FetchTokenAssociations,
    FetchCoinAssociations,
    FetchNftAssociations,
    FetchNftAssets,
    FetchAddressTransactions,
    FetchTransactions,
}

#[derive(Debug, Clone)]
pub struct ConsumerOptions {
    pub service: Option<ConsumerService>,
    pub indexer: Option<IndexerConsumer>,
}

#[cfg(test)]
mod tests {
    use super::IndexerConsumer;

    #[test]
    fn test_fetch_transactions_consumer_name() {
        assert_eq!("fetch_transactions".parse::<IndexerConsumer>(), Ok(IndexerConsumer::FetchTransactions));
    }
}
