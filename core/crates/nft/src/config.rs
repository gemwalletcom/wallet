use gem_client::RemoteProviderConfig;
use settings::Settings;

#[derive(Debug, Clone)]
pub(crate) struct OffchainClientConfig {
    pub(crate) timeout: u64,
    pub(crate) concurrency: usize,
    pub(crate) limit: usize,
}

#[derive(Clone)]
pub struct NFTProviderConfig {
    pub(crate) opensea: RemoteProviderConfig,
    pub(crate) magiceden: RemoteProviderConfig,
    pub(crate) ton: RemoteProviderConfig,
    pub(crate) offchain: OffchainClientConfig,
}

impl NFTProviderConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            opensea: settings.indexer.opensea.remote_provider_config(),
            magiceden: settings.indexer.magiceden.remote_provider_config(),
            ton: settings.indexer.ton.remote_provider_config(),
            offchain: OffchainClientConfig {
                timeout: settings.nft.offchain.timeout,
                concurrency: settings.nft.offchain.concurrency,
                limit: settings.nft.offchain.limit,
            },
        }
    }
}
