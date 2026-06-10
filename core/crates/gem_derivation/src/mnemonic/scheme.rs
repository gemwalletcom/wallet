use primitives::Chain;

use crate::AccountDerivationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DerivationScheme {
    Bip44,
    Bip84,
    Slip10,
    Cardano,
}

pub(super) fn derivation_scheme(chain: Chain) -> Result<DerivationScheme, AccountDerivationError> {
    match chain {
        Chain::Bitcoin | Chain::Litecoin => Ok(DerivationScheme::Bip84),
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
        | Chain::BitcoinCash
        | Chain::Thorchain
        | Chain::Mayachain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Tron
        | Chain::Doge
        | Chain::Zcash
        | Chain::Xrp
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei
        | Chain::Noble => Ok(DerivationScheme::Bip44),
        Chain::Solana | Chain::Ton | Chain::Aptos | Chain::Sui | Chain::Near | Chain::Stellar | Chain::Algorand | Chain::Polkadot => Ok(DerivationScheme::Slip10),
        Chain::Cardano => Ok(DerivationScheme::Cardano),
    }
}
