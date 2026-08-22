use gem_alchemy::{AlchemyApi, alchemy_url};
use gem_client::RemoteProviderConfig;
use primitives::Chain;
use settings::Settings;

#[derive(Debug, Clone)]
pub(crate) struct OffchainClientConfig {
    pub(crate) timeout: u64,
    pub(crate) concurrency: usize,
    pub(crate) limit: usize,
}

#[derive(Clone)]
pub struct NFTProviderConfig {
    pub(crate) alchemy: RemoteProviderConfig,
    pub(crate) opensea: RemoteProviderConfig,
    pub(crate) magiceden: RemoteProviderConfig,
    pub(crate) ton: RemoteProviderConfig,
    pub(crate) offchain: OffchainClientConfig,
}

impl NFTProviderConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let alchemy = settings.indexer.alchemy.remote_provider_config();
        let url = alchemy_url(Chain::SmartChain, &alchemy.url, AlchemyApi::Nft, &alchemy.key);

        Self {
            alchemy: RemoteProviderConfig { url, ..alchemy },
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
