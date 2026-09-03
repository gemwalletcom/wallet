use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::UdClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

const RECORD_KEYS: &[(Chain, &str)] = &[
    (Chain::Bitcoin, "crypto.BTC.address"),
    (Chain::Ethereum, "crypto.ETH.address"),
    (Chain::Solana, "crypto.SOL.address"),
    (Chain::Tron, "crypto.TRX.address"),
    (Chain::Cosmos, "crypto.ATOM.address"),
    (Chain::Doge, "crypto.DOGE.address"),
    (Chain::SmartChain, "crypto.BNB.version.BEP20.address"),
    (Chain::Polygon, "crypto.MATIC.version.MATIC.address"),
    (Chain::Optimism, "crypto.ETH.address"),
    (Chain::Arbitrum, "crypto.ETH.address"),
    (Chain::Base, "crypto.ETH.address"),
    (Chain::AvalancheC, "crypto.ETH.address"),
    (Chain::Aptos, "crypto.APT.address"),
];

const DOMAINS: &[&str] = &[
    "altimist",
    "anime",
    "austin",
    "binanceus",
    "bitcoin",
    "bitget",
    "blockchain",
    "clay",
    "crypto",
    "dao",
    "dfz",
    "farms",
    "go",
    "hi",
    "klever",
    "kresus",
    "kryptic",
    "manga",
    "metropolis",
    "nft",
    "pog",
    "polygon",
    "pudgy",
    "raiin",
    "secret",
    "smobler",
    "stepn",
    "tball",
    "ubu",
    "unstoppable",
    "wallet",
    "witg",
    "wrkx",
    "x",
    "888",
    "zil",
    "ca",
    "com",
    "pw",
    "eth",
];

pub struct UdProvider {
    client: UdClient,
}

impl UdProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: UdClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for UdProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Ud
    }

    fn domains(&self) -> Vec<&'static str> {
        DOMAINS.to_vec()
    }

    fn chains(&self) -> Vec<Chain> {
        RECORD_KEYS.iter().map(|(chain, _)| *chain).collect()
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let key = RECORD_KEYS
            .iter()
            .find_map(|(candidate, key)| (*candidate == chain).then_some(*key))
            .ok_or(format!("unsupported chain: {chain}"))?;
        let domain = self.client.get_domain(&query.domain).await?;
        Ok(domain.records.get(key).cloned())
    }
}
