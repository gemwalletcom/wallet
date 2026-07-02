use std::error::Error;

use base64::{Engine as _, engine::general_purpose};
use primitives::Chain;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use super::model::ZerionPositionsResponse;

pub struct ZerionClient {
    url: String,
    client: reqwest::Client,
}

impl ZerionClient {
    pub fn new(url: String, key: String) -> Self {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Basic {}", general_purpose::STANDARD.encode(format!("{key}:")));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());

        Self {
            url,
            client: reqwest::Client::builder().default_headers(headers).build().unwrap(),
        }
    }

    pub fn chain_id(chain: &Chain) -> Result<&str, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::SmartChain => Ok("binance-smart-chain"),
            Chain::AvalancheC => Ok("avalanche"),
            Chain::Gnosis => Ok("xdai"),
            Chain::ZkSync => Ok("zksync-era"),
            Chain::Ethereum | Chain::Polygon | Chain::Arbitrum | Chain::Optimism | Chain::Base | Chain::Fantom | Chain::Linea | Chain::Celo => Ok(chain.as_ref()),
            _ => Err(format!("Unsupported chain for Zerion: {:?}", chain).into()),
        }
    }

    pub async fn get_wallet_positions(&self, chain: Chain, address: &str) -> Result<ZerionPositionsResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v1/wallets/{}/positions/", self.url, address);
        let chain_id = Self::chain_id(&chain)?;
        let query = [
            ("filter[positions]", "only_complex"),
            ("filter[chain_ids]", chain_id),
            ("filter[trash]", "only_non_trash"),
            ("currency", "usd"),
            ("sort", "-value"),
        ];

        Ok(self.client.get(url).query(&query).send().await?.error_for_status()?.json().await?)
    }
}
