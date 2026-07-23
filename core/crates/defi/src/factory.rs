use std::{collections::HashMap, sync::Arc};

use base64::{Engine as _, engine::general_purpose};
use gem_client::ReqwestClient;

use crate::config::DefiProviderConfig;
use crate::provider::DefiProvider;
use crate::providers::{DeBankClient, JupiterClient, ZerionClient};

pub struct DefiProviderFactory;

impl DefiProviderFactory {
    pub fn new_providers(config: DefiProviderConfig) -> Vec<Arc<dyn DefiProvider>> {
        let client = ReqwestClient::new(String::new(), gem_client::reqwest_client());
        let zerion_client = config.zerion.configure_client(client.clone()).with_default_headers(HashMap::from([(
            "Authorization".to_string(),
            format!("Basic {}", general_purpose::STANDARD.encode(format!("{}:", config.zerion.key))),
        )]));
        let jupiter_client = config
            .jupiter
            .configure_client(client)
            .with_default_headers(HashMap::from([("x-api-key".to_string(), config.jupiter.key)]));
        vec![
            Arc::new(ZerionClient::new(zerion_client)),
            Arc::new(JupiterClient::new_with_client(jupiter_client)),
            Arc::new(DeBankClient),
        ]
    }
}
