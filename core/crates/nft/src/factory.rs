use std::{collections::HashMap, sync::Arc};

use gem_client::ReqwestClient;
use gem_ton::rpc::client::TonClient;

use crate::config::NFTProviderConfig;
use crate::provider::NFTProvider;
use crate::providers::ton::provider::TonNftProvider;
use crate::providers::{MagicEdenEvmClient, MagicEdenSolanaClient, OpenSeaClient};

pub struct NFTProviderFactory;

impl NFTProviderFactory {
    pub fn new_providers(config: NFTProviderConfig) -> Vec<Arc<dyn NFTProvider>> {
        let client = ReqwestClient::new(String::new(), gem_client::reqwest_client());
        let opensea_client = config
            .opensea
            .configure_client(client.clone())
            .with_default_headers(HashMap::from([("x-api-key".to_string(), config.opensea.key)]));
        let magiceden_client = config
            .magiceden
            .configure_client(client.clone())
            .with_default_headers(HashMap::from([("Authorization".to_string(), format!("Bearer {}", config.magiceden.key))]));
        let ton_client = config.ton.configure_client(client);

        vec![
            Arc::new(OpenSeaClient::new(opensea_client)),
            Arc::new(MagicEdenSolanaClient::new(magiceden_client.clone())),
            Arc::new(MagicEdenEvmClient::new(magiceden_client)),
            Arc::new(TonNftProvider::new(TonClient::new(ton_client), config.offchain)),
        ]
    }
}
