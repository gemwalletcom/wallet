use std::{collections::HashMap, sync::Arc};

use alchemy::{AlchemyApi, alchemy_url};
use blockscout::Client as BlockscoutClient;
use gem_client::RemoteProviderConfig;
use gem_reqwest::ReqwestClient;
use gem_ton::rpc::client::TonClient;
use primitives::{EVMChain, NFTChain};

use crate::config::NFTProviderConfig;
use crate::provider::NFTProvider;
use crate::providers::ton::provider::TonNftProvider;
use crate::providers::{AlchemyClient, AlchemyProvider, BlockscoutProvider, MagicEdenSolanaClient, OpenSeaClient};

pub struct NFTProviderFactory;

impl NFTProviderFactory {
    pub fn new_providers(config: NFTProviderConfig) -> Vec<Arc<dyn NFTProvider>> {
        let client = ReqwestClient::new(String::new(), gem_reqwest::reqwest_client());
        let opensea_client = client.clone().with_base_url(config.opensea.url).with_default_headers(HashMap::from([("x-api-key".to_string(), config.opensea.key)]));
        let magiceden_client = client
            .clone()
            .with_base_url(config.magiceden.url)
            .with_default_headers(HashMap::from([("Authorization".to_string(), format!("Bearer {}", config.magiceden.key))]));
        let blockscout_client = BlockscoutClient::new(client.clone().with_base_url(config.blockscout.url), EVMChain::Arc.chain_id(), config.blockscout.key);
        let ton_client = client.clone().with_base_url(config.ton.url);

        vec![
            Arc::new(OpenSeaClient::new(opensea_client)),
            Arc::new(MagicEdenSolanaClient::new(magiceden_client)),
            Arc::new(BlockscoutProvider::new(blockscout_client, NFTChain::Arc)),
            Arc::new(Self::alchemy_provider(&config.alchemy, &client, NFTChain::SmartChain)),
            Arc::new(Self::alchemy_provider(&config.alchemy, &client, NFTChain::Arc)),
            Arc::new(TonNftProvider::new(TonClient::new(ton_client), config.offchain)),
        ]
    }

    pub(crate) fn alchemy_provider(config: &RemoteProviderConfig, client: &ReqwestClient, chain: NFTChain) -> AlchemyProvider<ReqwestClient> {
        let url = alchemy_url(chain.into(), &config.url, AlchemyApi::Nft, &config.key);
        AlchemyProvider::new(AlchemyClient::new(client.clone().with_base_url(url)), chain)
    }
}
