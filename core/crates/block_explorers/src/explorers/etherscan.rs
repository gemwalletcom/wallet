use crate::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata};
use crate::explorers::{BlockScout, OkxExplorer, RouteScan, TempoExplorer};
use primitives::EVMChain;

pub struct EtherScan;

impl EtherScan {
    pub fn boxed(chain: EVMChain) -> Box<dyn BlockExplorer> {
        match chain {
            EVMChain::Ethereum => Explorer::boxed(Metadata::with_token("Etherscan", "https://etherscan.io")),
            EVMChain::SmartChain => Explorer::boxed(Metadata::with_token("BscScan", "https://bscscan.com")),
            EVMChain::Polygon => Explorer::boxed(Metadata::with_token("PolygonScan", "https://polygonscan.com")),
            EVMChain::Plasma => Explorer::boxed(Metadata::with_token("PlasmaScan", "https://plasmascan.to")),
            EVMChain::Arbitrum => Explorer::boxed(Metadata::with_token("ArbiScan", "https://arbiscan.io")),
            EVMChain::Optimism => Explorer::boxed(Metadata::with_token("Etherscan", "https://optimistic.etherscan.io")),
            EVMChain::Base => Explorer::boxed(Metadata::with_token("BaseScan", "https://basescan.org")),
            EVMChain::AvalancheC => Explorer::boxed(Metadata::with_token("SnowScan", "https://snowscan.xyz")),
            EVMChain::OpBNB => Explorer::boxed(Metadata::with_token("opBNBScan", "https://opbnb.bscscan.com")),
            EVMChain::Fantom => Explorer::boxed(Metadata::new("Fantom Explorer", "https://explorer.fantom.network")),
            EVMChain::Gnosis => Explorer::boxed(Metadata::with_token("GnosisScan", "https://gnosisscan.io")),
            EVMChain::Manta => BlockScout::new_manta(),
            EVMChain::Blast => Explorer::boxed(Metadata::with_token("BlastScan", "https://blastscan.io")),
            EVMChain::Linea => Explorer::boxed(Metadata::with_token("LineaScan", "https://lineascan.build")),
            EVMChain::ZkSync => Explorer::boxed(Metadata::with_token("zkSync Era Explorer", "https://era.zksync.network")),
            EVMChain::Celo => Explorer::boxed(Metadata::with_token("CeloScan", "https://celoscan.io")),
            EVMChain::Mantle => Explorer::boxed(Metadata::with_token("MantleScan", "https://mantlescan.xyz")),
            EVMChain::World => Explorer::boxed(Metadata::with_token("WorldScan", "https://worldscan.org")),
            EVMChain::Sonic => Explorer::boxed(Metadata::with_token("SonicScan", "https://sonicscan.org")),
            EVMChain::SeiEvm => Explorer::boxed(Metadata::with_token("Seiscan", "https://seiscan.io")),
            EVMChain::Abstract => Explorer::boxed(Metadata::with_token("Abscan", "https://abscan.org")),
            EVMChain::Berachain => Explorer::boxed(Metadata::with_token("Berascan", "https://berascan.com")),
            EVMChain::Unichain => Explorer::boxed(Metadata::with_token("Uniscan", "https://uniscan.xyz")),
            EVMChain::Monad => Explorer::boxed(Metadata::with_token("Monadscan", "https://monadscan.com")),
            EVMChain::Hyperliquid => Explorer::boxed(Metadata::with_token("HyperEvmScan", "https://hyperevmscan.io")),
            EVMChain::Robinhood => Explorer::boxed(Metadata::with_token("RobinScan", "https://robin.etherscan.io")),
            EVMChain::Stable => Explorer::boxed(Metadata::with_token("Stablescan", "https://stablescan.xyz")),
            EVMChain::Tempo => TempoExplorer::boxed(),
            EVMChain::Ink => RouteScan::new_ink(),
            EVMChain::Arc => Explorer::boxed(Metadata::with_token("ArcScan", "https://arc.etherscan.io")),
            EVMChain::XLayer => OkxExplorer::new_xlayer(),
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain};

    use super::*;

    #[test]
    fn test_etherscan_urls() {
        let explorer = EtherScan::boxed(EVMChain::Ethereum);

        assert_eq!(
            explorer.get_address_url("0x1f9090aae28b8a3dceadf281b0f12828e676c326"),
            "https://etherscan.io/address/0x1f9090aae28b8a3dceadf281b0f12828e676c326"
        );
        assert_eq!(
            explorer.get_tx_url("0xfd96a9ee20a7440bf65a5b8ecf7f884289ed78e28f82d45343a70f459e7a42a0"),
            "https://etherscan.io/tx/0xfd96a9ee20a7440bf65a5b8ecf7f884289ed78e28f82d45343a70f459e7a42a0"
        );
        assert_eq!(
            explorer.get_token_url(&AssetId::from_token(Chain::Ethereum, "0xdac17f958d2ee523a2206206994597c13d831ec7")),
            Some("https://etherscan.io/token/0xdac17f958d2ee523a2206206994597c13d831ec7".to_string())
        );
        assert_eq!(
            explorer.get_nft_url("0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee", "11871"),
            Some("https://etherscan.io/nft/0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee/11871".to_string())
        );
    }
}
