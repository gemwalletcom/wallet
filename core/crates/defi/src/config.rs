use gem_client::RemoteProviderConfig;
use settings::Settings;

#[derive(Clone)]
pub struct DefiProviderConfig {
    pub(crate) zerion: RemoteProviderConfig,
    pub(crate) jupiter: RemoteProviderConfig,
}

impl DefiProviderConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            zerion: settings.indexer.zerion.remote_provider_config(),
            jupiter: settings.indexer.jupiter.remote_provider_config(),
        }
    }
}
