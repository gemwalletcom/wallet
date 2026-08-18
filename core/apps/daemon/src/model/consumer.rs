use strum::{AsRefStr, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(ConsumerServiceKind))]
#[strum_discriminants(derive(EnumString, IntoStaticStr))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ConsumerService {
    Store,
    Indexer(IndexerService),
    Notifications,
    Rewards,
    Support,
    Fiat,
}

impl ConsumerService {
    pub fn all() -> Vec<Self> {
        std::iter::once(Self::Store)
            .chain(IndexerService::iter().map(Self::Indexer))
            .chain([Self::Notifications, Self::Rewards, Self::Support, Self::Fiat])
            .collect()
    }

    pub fn name(self) -> String {
        let name: &'static str = ConsumerServiceKind::from(self).into();
        match self {
            Self::Indexer(service) => format!("{} {}", name, service.as_ref()),
            _ => name.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum IndexerService {
    Assets,
    Nfts,
    Balances,
    Blocks,
    Transactions,
}

impl IndexerService {
    pub fn consumers(self) -> &'static [IndexerConsumer] {
        match self {
            Self::Assets => &[IndexerConsumer::FetchAssets, IndexerConsumer::FetchLists, IndexerConsumer::FetchPrices],
            Self::Nfts => &[IndexerConsumer::FetchNftAssociations, IndexerConsumer::FetchNftAssets],
            Self::Balances => &[IndexerConsumer::FetchTokenAssociations, IndexerConsumer::FetchCoinAssociations],
            Self::Blocks => &[IndexerConsumer::FetchBlocks],
            Self::Transactions => &[IndexerConsumer::FetchAddressTransactions, IndexerConsumer::FetchTransactions],
        }
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

impl ConsumerOptions {
    pub fn parse(parts: &[&str]) -> Result<Self, String> {
        let Some(name) = parts.first() else {
            return Ok(Self { service: None, indexer: None });
        };
        let kind = name.parse::<ConsumerServiceKind>().map_err(|_| format!("Invalid consumer: {name}"))?;

        let service = match kind {
            ConsumerServiceKind::Indexer => return Self::parse_indexer(&parts[1..]),
            ConsumerServiceKind::Store => ConsumerService::Store,
            ConsumerServiceKind::Notifications => ConsumerService::Notifications,
            ConsumerServiceKind::Rewards => ConsumerService::Rewards,
            ConsumerServiceKind::Support => ConsumerService::Support,
            ConsumerServiceKind::Fiat => ConsumerService::Fiat,
        };
        Ok(Self {
            service: Some(service),
            indexer: None,
        })
    }

    fn parse_indexer(parts: &[&str]) -> Result<Self, String> {
        let name = parts.first().ok_or_else(|| "Missing indexer service".to_owned())?;
        let service = name.parse::<IndexerService>().map_err(|_| format!("Invalid indexer service: {name}"))?;
        let indexer = parts
            .get(1)
            .map(|name| name.parse::<IndexerConsumer>().map_err(|_| format!("Invalid indexer consumer: {name}")))
            .transpose()?;
        if let Some(indexer) = indexer
            && !service.consumers().contains(&indexer)
        {
            return Err(format!("Indexer consumer {} does not belong to {}", indexer.as_ref(), service.as_ref()));
        }
        Ok(Self {
            service: Some(ConsumerService::Indexer(service)),
            indexer,
        })
    }

    pub fn name(&self) -> String {
        let service = self.service.map(ConsumerService::name).unwrap_or_else(|| "all".to_owned());
        match self.indexer {
            Some(indexer) => format!("{} {}", service, indexer.as_ref()),
            None => service,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConsumerService, IndexerConsumer, IndexerService};

    #[test]
    fn test_indexer_consumer_names() {
        assert_eq!("fetch_transactions".parse::<IndexerConsumer>(), Ok(IndexerConsumer::FetchTransactions));
        assert_eq!("assets".parse::<IndexerService>(), Ok(IndexerService::Assets));
        assert_eq!(ConsumerService::Indexer(IndexerService::Assets).name(), "indexer assets");
        assert_eq!(
            IndexerService::Assets.consumers(),
            &[IndexerConsumer::FetchAssets, IndexerConsumer::FetchLists, IndexerConsumer::FetchPrices]
        );
        assert_eq!(IndexerService::Blocks.consumers(), &[IndexerConsumer::FetchBlocks]);
    }
}
