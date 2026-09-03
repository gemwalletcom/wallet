use std::error::Error;

use gem_client::{Client, ClientExt};
use primitives::Chain;

use super::model::{PositionsQuery, ZerionPositionsResponse};
use super::target::ZerionTarget;

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
        let target = ZerionTarget::WalletPositions {
            address: address.to_string(),
            query: PositionsQuery::complex(chain_id(&chain)?),
        };
        Ok(self.client.get(target).await?)
    }
}
