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
    FetchPrices,
    FetchBlocks,
    FetchTokenAssociations,
    FetchCoinAssociations,
    FetchNftAssociations,
    FetchNftAssets,
    FetchAddressTransactions,
}

#[derive(Debug, Clone)]
pub struct ConsumerOptions {
    pub service: Option<ConsumerService>,
    pub indexer: Option<IndexerConsumer>,
}
