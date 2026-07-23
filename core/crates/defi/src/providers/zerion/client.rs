use std::error::Error;

use gem_client::{Client, ClientExt};
use primitives::Chain;

use super::model::ZerionPositionsResponse;

pub struct ZerionClient<C: Client> {
    client: C,
}

pub(super) fn chain_id(chain: &Chain) -> Result<&str, Box<dyn Error + Send + Sync>> {
    match chain {
        Chain::SmartChain => Ok("binance-smart-chain"),
        Chain::AvalancheC => Ok("avalanche"),
        Chain::Gnosis => Ok("xdai"),
        Chain::ZkSync => Ok("zksync-era"),
        Chain::Ethereum | Chain::Polygon | Chain::Arbitrum | Chain::Optimism | Chain::Base | Chain::Fantom | Chain::Linea | Chain::Celo => Ok(chain.as_ref()),
        _ => Err(format!("Unsupported chain for Zerion: {:?}", chain).into()),
    }
}

impl<C: Client> ZerionClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_wallet_positions(&self, chain: Chain, address: &str) -> Result<ZerionPositionsResponse, Box<dyn Error + Send + Sync>> {
        let chain_id = chain_id(&chain)?;
        let query = [
            ("filter[positions]".to_string(), "only_complex".to_string()),
            ("filter[chain_ids]".to_string(), chain_id.to_string()),
            ("filter[trash]".to_string(), "only_non_trash".to_string()),
            ("currency".to_string(), "usd".to_string()),
            ("sort".to_string(), "-value".to_string()),
        ];
        Ok(self.client.get_with_query(&format!("/v1/wallets/{address}/positions/"), &query).await?)
    }
}
