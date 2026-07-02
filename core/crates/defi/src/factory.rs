use std::sync::Arc;

use crate::config::DefiProviderConfig;
use crate::provider::DefiProvider;
use crate::providers::{DeBankClient, ZerionClient};

pub struct DefiProviderFactory;

impl DefiProviderFactory {
    pub fn new_providers(config: DefiProviderConfig) -> Vec<Arc<dyn DefiProvider>> {
        vec![Arc::new(ZerionClient::new(config.zerion_url, config.zerion_key)), Arc::new(DeBankClient)]
    }
}
