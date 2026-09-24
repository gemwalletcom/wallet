mod explorers;

use std::str::FromStr;

use primitives::{Chain, EVMChain, SwapProvider};

use crate::explorers::{
    AcrossScan, AlgorandAllo, AlgorandPera, BlockScout, BlockVision, Blocksec, Cardanocan, ChainflipScan, EtherScan, Explorer, FlowScan, HyperliquidExplorer, HypurrScan, MantleExplorer, MayaScan, MayanScan, Metadata, NearBlocks,
    NearIntents, OkxExplorer, RelayScan, RouteScan, RuneScan, SkipExplorer, SubScan, SwapsXyzScan, TempoExplorer, TonScan, TronScan, Viewblock, XrpScan, ZkSync, aptos, blockchair, mempool, mintscan, solana, stellar_expert, sui, threexpl,
    ton,
};

#[derive(Debug, Default, Clone)]
pub struct ExplorerInput {
    pub hash: String,
    pub recipient: Option<String>,
    pub memo: Option<String>,
}

impl ExplorerInput {
    pub fn new_recipient(recipient: impl Into<String>) -> Self {
        Self {
            hash: String::new(),
            recipient: Some(recipient.into()),
            memo: None,
        }
    }

    pub fn new_memo(recipient: impl Into<String>, memo: impl Into<String>) -> Self {
        Self {
            hash: String::new(),
            recipient: Some(recipient.into()),
            memo: Some(memo.into()),
        }
    }
}

impl<T: Into<String>> From<T> for ExplorerInput {
    fn from(hash: T) -> Self {
        Self {
            hash: hash.into(),
            recipient: None,
            memo: None,
        }
    }
}

pub trait BlockExplorer: Send + Sync {
    fn name(&self) -> String;
    fn get_tx_url(&self, hash: &str) -> String;
    fn get_address_url(&self, address: &str) -> String;
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
    fn get_nft_url(&self, _contract: &str, _token_id: &str) -> Option<String> {
        None
    }
    fn get_validator_url(&self, _validator: &str) -> Option<String> {
        None
    }

    fn get_swap_tx_url(&self, input: &ExplorerInput) -> String {
        self.get_tx_url(&input.hash)
    }

    fn new() -> Box<Self>
    where
        Self: Default + Sized,
    {
        Box::new(Self::default())
    }
}

pub fn get_block_explorers_by_chain(chain: &str) -> Vec<Box<dyn BlockExplorer>> {
    let Ok(chain) = Chain::from_str(chain) else {
        return vec![];
    };
    get_block_explorers(chain)
}

pub fn get_block_explorer(chain: Chain, name: &str) -> Box<dyn BlockExplorer> {
    let mut explorers = get_block_explorers(chain);
    let index = explorers.iter().position(|explorer| explorer.name() == name).unwrap_or_default();
    explorers.swap_remove(index)
}

pub fn get_block_explorers(chain: Chain) -> Vec<Box<dyn BlockExplorer>> {
    match chain {
        Chain::Bitcoin => vec![blockchair::new_bitcoin(), mempool::new(), threexpl::new_bitcoin()],
        Chain::BitcoinCash => vec![blockchair::new_bitcoin_cash(), threexpl::new_bitcoin_cash()],
        Chain::Litecoin => vec![blockchair::new_litecoin(), threexpl::new_litecoin()],
        Chain::Doge => vec![blockchair::new_doge(), threexpl::new_doge()],
        Chain::Dash => vec![Explorer::boxed(Metadata::new("Dash Explorer", "https://explorer.dash.org/insight"))],
        Chain::Zcash => vec![blockchair::new_zcash(), threexpl::new_zcash()],

        Chain::Ethereum => vec![EtherScan::boxed(EVMChain::Ethereum), blockchair::new_ethereum(), Blocksec::new_ethereum()],
        Chain::SmartChain => vec![EtherScan::boxed(EVMChain::SmartChain), blockchair::new_bnb(), Blocksec::new_bsc()],
        Chain::Polygon => vec![EtherScan::boxed(EVMChain::Polygon), blockchair::new_polygon(), Blocksec::new_polygon()],
        Chain::Arbitrum => vec![EtherScan::boxed(EVMChain::Arbitrum), blockchair::new_arbitrum(), Blocksec::new_arbitrum()],
        Chain::Optimism => vec![EtherScan::boxed(EVMChain::Optimism), blockchair::new_optimism(), Blocksec::new_optimism()],
        Chain::Base => vec![EtherScan::boxed(EVMChain::Base), blockchair::new_base(), Blocksec::new_base()],
        Chain::AvalancheC => vec![EtherScan::boxed(EVMChain::AvalancheC), RouteScan::new_avax(), blockchair::new_avalanche()],
        Chain::OpBNB => vec![EtherScan::boxed(EVMChain::OpBNB), blockchair::new_opbnb()],
        Chain::Fantom => vec![EtherScan::boxed(EVMChain::Fantom), blockchair::new_fantom()],
        Chain::Gnosis => vec![EtherScan::boxed(EVMChain::Gnosis), blockchair::new_gnosis()],
        Chain::Manta => vec![BlockScout::new_manta(), EtherScan::boxed(EVMChain::Manta)],
        Chain::Blast => vec![EtherScan::boxed(EVMChain::Blast)],
        Chain::Linea => vec![EtherScan::boxed(EVMChain::Linea), blockchair::new_linea()],
        Chain::Celo => vec![BlockScout::new_celo(), EtherScan::boxed(EVMChain::Celo)],
        Chain::ZkSync => vec![ZkSync::boxed(), EtherScan::boxed(EVMChain::ZkSync)],
        Chain::World => vec![EtherScan::boxed(EVMChain::World)],
        Chain::Plasma => vec![EtherScan::boxed(EVMChain::Plasma)],
        Chain::Solana => vec![solana::new_solscan(), solana::new_solana_fm(), blockchair::new_solana()],
        Chain::Thorchain => vec![RuneScan::boxed(), Viewblock::boxed()],
        Chain::Mayachain => vec![MayaScan::boxed()],

        Chain::Cosmos => vec![mintscan::new_cosmos()],
        Chain::Osmosis => vec![mintscan::new_osmosis()],
        Chain::Celestia => vec![mintscan::new_celestia()],
        Chain::Injective => vec![mintscan::new_injective()],
        Chain::Sei => vec![mintscan::new_sei()],
        Chain::SeiEvm => vec![Explorer::boxed(Metadata::with_token("Seiscan", "https://seiscan.io"))],
        Chain::Noble => vec![mintscan::new_noble()],
        Chain::Mantle => vec![MantleExplorer::boxed(), EtherScan::boxed(EVMChain::Mantle)],

        Chain::Ton => vec![ton::new_ton_viewer(), TonScan::boxed(), blockchair::new_ton()],
        Chain::Tron => vec![TronScan::boxed(), blockchair::new_tron()],
        Chain::Xrp => vec![XrpScan::boxed(), blockchair::new_xrp()],
        Chain::Aptos => vec![aptos::new_aptos_scan(), aptos::new_aptos_explorer(), blockchair::new_aptos()],
        Chain::Sui => vec![sui::new_sui_scan(), BlockVision::new_sui()],
        Chain::Near => vec![NearBlocks::boxed()],
        Chain::Stellar => vec![stellar_expert::new(), blockchair::new_stellar()],
        Chain::Sonic => vec![EtherScan::boxed(EVMChain::Sonic), RouteScan::new_sonic()],
        Chain::Algorand => vec![AlgorandAllo::boxed(), AlgorandPera::boxed()],
        Chain::Polkadot => vec![SubScan::new_polkadot(), blockchair::new_polkadot()],
        Chain::Cardano => vec![Cardanocan::boxed()],
        Chain::Abstract => vec![EtherScan::boxed(EVMChain::Abstract)],
        Chain::Berachain => vec![EtherScan::boxed(EVMChain::Berachain)],
        Chain::Ink => vec![RouteScan::new_ink(), BlockScout::new_ink(), OkxExplorer::new_ink()],
        Chain::Unichain => vec![EtherScan::boxed(EVMChain::Unichain)],
        Chain::Hyperliquid => vec![EtherScan::boxed(EVMChain::Hyperliquid), BlockScout::new_hyperliquid()],
        Chain::HyperCore => vec![HyperliquidExplorer::boxed(), HypurrScan::boxed(), FlowScan::boxed()],
        Chain::Monad => vec![EtherScan::boxed(EVMChain::Monad), BlockVision::new_monad()],
        Chain::XLayer => vec![OkxExplorer::new_xlayer()],
        Chain::Robinhood => vec![EtherScan::boxed(EVMChain::Robinhood), BlockScout::new_robinhood()],
        Chain::Stable => vec![EtherScan::boxed(EVMChain::Stable)],
        Chain::Tempo => vec![TempoExplorer::boxed()],
        Chain::Arc => vec![BlockScout::new_arc(), EtherScan::boxed(EVMChain::Arc)],
    }
}

pub fn swap_explorer(provider: SwapProvider, chain: Chain) -> Option<Box<dyn BlockExplorer>> {
    match provider {
        SwapProvider::Mayan => Some(MayanScan::boxed()),
        SwapProvider::Thorchain => Some(RuneScan::boxed()),
        SwapProvider::Mayachain => Some(MayaScan::boxed()),
        SwapProvider::Across => Some(AcrossScan::boxed()),
        SwapProvider::Chainflip => Some(ChainflipScan::boxed()),
        SwapProvider::NearIntents => Some(NearIntents::boxed()),
        SwapProvider::Relay => Some(RelayScan::boxed()),
        SwapProvider::Squid => Some(SkipExplorer::boxed(chain)),
        SwapProvider::SwapsXyz => Some(SwapsXyzScan::boxed()),
        SwapProvider::UniswapV3
        | SwapProvider::UniswapV4
        | SwapProvider::PancakeswapV3
        | SwapProvider::Panora
        | SwapProvider::Jupiter
        | SwapProvider::Okx
        | SwapProvider::Oku
        | SwapProvider::Wagmi
        | SwapProvider::CetusClmm
        | SwapProvider::StonfiV2
        | SwapProvider::Aerodrome
        | SwapProvider::Hyperliquid
        | SwapProvider::Orca => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swaps_xyz_explorer() {
        let transaction_id = "0x6331c6eded7cfe4ed578e41a57855102b3fd60b3daa2c4bef992f4f5869856b4";
        for chain in [Chain::Ton, Chain::Algorand, Chain::Stellar] {
            let explorer = swap_explorer(SwapProvider::SwapsXyz, chain).unwrap();
            assert_eq!(explorer.name(), "Swaps.xyz");
            assert_eq!(explorer.get_swap_tx_url(&transaction_id.into()), format!("https://scan.swaps.xyz/transactions?search={transaction_id}"));
        }
    }
}
