use gem_client::RemoteProviderConfig;
use primitives::Chain;
use settings::Settings;

#[derive(Clone, Default)]
pub(crate) struct IndexerProvidersConfig {
    pub(crate) alchemy: RemoteProviderConfig,
    pub(crate) algorand: RemoteProviderConfig,
    pub(crate) ankr: RemoteProviderConfig,
    pub(crate) blockscout: RemoteProviderConfig,
    pub(crate) fastnear: RemoteProviderConfig,
    pub(crate) subscan: RemoteProviderConfig,
    pub(crate) sui: RemoteProviderConfig,
    pub(crate) trongrid: RemoteProviderConfig,
}

#[derive(Clone)]
pub struct ProviderConfig {
    pub(crate) chain: Chain,
    pub(crate) url: String,
    pub(crate) indexers: IndexerProvidersConfig,
}

impl ProviderConfig {
    pub fn new(chain: Chain, url: &str) -> Self {
        Self {
            chain,
            url: url.to_string(),
            indexers: IndexerProvidersConfig::default(),
        }
    }

    pub(crate) fn from_settings(chain: Chain, url: &str, settings: &Settings) -> Self {
        Self {
            chain,
            url: url.to_string(),
            indexers: IndexerProvidersConfig {
                alchemy: settings.indexer.alchemy.remote_provider_config(),
                algorand: settings.indexer.algorand.remote_provider_config(),
                ankr: settings.indexer.ankr.remote_provider_config(),
                blockscout: settings.indexer.blockscout.remote_provider_config(),
                fastnear: settings.indexer.fastnear.remote_provider_config(),
                subscan: settings.indexer.subscan.remote_provider_config(),
                sui: settings.indexer.sui.remote_provider_config(),
                trongrid: settings.indexer.trongrid.remote_provider_config(),
            },
        }
    }
}
