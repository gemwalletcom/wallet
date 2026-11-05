use std::{collections::HashMap, sync::Arc};

use gem_client::ReqwestClient;
use gem_encoding::encode_base64;

use crate::config::DefiProviderConfig;
use crate::provider::DefiProvider;
use crate::providers::{JupiterClient, ZerionClient};

pub struct DefiProviderFactory;

impl DefiProviderFactory {
    pub fn new_providers(config: DefiProviderConfig) -> Vec<Arc<dyn DefiProvider>> {
        let client = ReqwestClient::new(String::new(), gem_client::reqwest_client());
        let zerion_client = config
            .zerion
            .configure_client(client.clone())
            .with_default_headers(HashMap::from([("Authorization".to_string(), format!("Basic {}", encode_base64(format!("{}:", config.zerion.key).as_bytes())))]));
        let jupiter_client = config.jupiter.configure_client(client).with_default_headers(HashMap::from([("x-api-key".to_string(), config.jupiter.key)]));
        vec![Arc::new(ZerionClient::new(zerion_client)), Arc::new(JupiterClient::new_with_client(jupiter_client))]
    }
}
