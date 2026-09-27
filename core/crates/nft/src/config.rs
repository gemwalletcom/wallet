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
    pub(crate) alchemy: RemoteProviderConfig,
    pub(crate) blockscout: RemoteProviderConfig,
    pub(crate) opensea: RemoteProviderConfig,
    pub(crate) magiceden: RemoteProviderConfig,
    pub(crate) ton: RemoteProviderConfig,
    pub(crate) offchain: OffchainClientConfig,
}

impl NFTProviderConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            alchemy: settings.nft.alchemy.remote_provider_config(),
            blockscout: settings.nft.blockscout.remote_provider_config(),
            opensea: settings.nft.opensea.remote_provider_config(),
            magiceden: settings.nft.magiceden.remote_provider_config(),
            ton: settings.nft.ton.remote_provider_config(),
            offchain: OffchainClientConfig {
                timeout: settings.nft.offchain.timeout,
                concurrency: settings.nft.offchain.concurrency,
                limit: settings.nft.offchain.limit,
            },
        }
    }
}
