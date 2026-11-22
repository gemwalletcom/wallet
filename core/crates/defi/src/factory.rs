use std::{collections::HashMap, sync::Arc};

use gem_encoding::encode_base64;
use gem_reqwest::ReqwestClient;

use crate::config::DefiProviderConfig;
use crate::provider::DefiProvider;
use crate::providers::{JupiterClient, ZerionClient};

pub struct DefiProviderFactory;

impl DefiProviderFactory {
    pub fn new_providers(config: DefiProviderConfig) -> Vec<Arc<dyn DefiProvider>> {
        let client = ReqwestClient::new(String::new(), gem_reqwest::reqwest_client());
        let zerion_client = client
            .clone()
            .with_base_url(config.zerion.url)
            .with_default_headers(HashMap::from([("Authorization".to_string(), format!("Basic {}", encode_base64(format!("{}:", config.zerion.key).as_bytes())))]));
        let jupiter_client = client.with_base_url(config.jupiter.url).with_default_headers(HashMap::from([("x-api-key".to_string(), config.jupiter.key)]));
        vec![Arc::new(ZerionClient::new(zerion_client)), Arc::new(JupiterClient::new_with_client(jupiter_client))]
    }
}
