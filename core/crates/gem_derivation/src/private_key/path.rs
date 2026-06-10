use primitives::Chain;

pub fn default_derivation_path(chain: Chain) -> &'static str {
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis
        | Chain::Manta
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo
        | Chain::World
        | Chain::Sonic
        | Chain::SeiEvm
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink
        | Chain::Unichain
        | Chain::Hyperliquid
        | Chain::HyperCore
        | Chain::Monad
        | Chain::Plasma
        | Chain::XLayer
        | Chain::Stable
        | Chain::Injective => "m/44'/60'/0'/0/0",
        Chain::Bitcoin => "m/84'/0'/0'/0/0",
        Chain::BitcoinCash => "m/44'/145'/0'/0/0",
        Chain::Litecoin => "m/84'/2'/0'/0/0",
        Chain::Thorchain | Chain::Mayachain => "m/44'/931'/0'/0/0",
        Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Noble | Chain::Sei => "m/44'/118'/0'/0/0",
        Chain::Solana => "m/44'/501'/0'/0'",
        Chain::Ton => "m/44'/607'/0'",
        Chain::Tron => "m/44'/195'/0'/0/0",
        Chain::Doge => "m/44'/3'/0'/0/0",
        Chain::Zcash => "m/44'/133'/0'/0/0",
        Chain::Aptos => "m/44'/637'/0'/0'/0'",
        Chain::Sui => "m/44'/784'/0'/0'/0'",
        Chain::Xrp => "m/44'/144'/0'/0/0",
        Chain::Near => "m/44'/397'/0'",
        Chain::Stellar => "m/44'/148'/0'",
        Chain::Algorand => "m/44'/283'/0'/0'/0'",
        Chain::Polkadot => "m/44'/354'/0'/0'/0'",
        Chain::Cardano => "m/1852'/1815'/0'/0/0",
    }
}
