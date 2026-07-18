use primitives::Chain;

#[derive(Clone, Copy)]
pub(crate) struct NodeFixture {
    pub(crate) addresses: &'static [&'static str],
    pub(crate) transaction_hashes: &'static [&'static str],
}

pub(crate) fn fixture(chain: Chain) -> Option<NodeFixture> {
    match chain {
        Chain::Ethereum => Some(NodeFixture {
            addresses: &["0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"],
            transaction_hashes: &["0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3"],
        }),
        Chain::SmartChain => Some(NodeFixture {
            addresses: &["0x2A49C84B7173e21f9116B2798735f87531526b36"],
            transaction_hashes: &["0xa9f6e1d1a02ba5bb5aa9b3c83773ef9ac6d8fe9abb1fa4512d422f0194d5d833"],
        }),
        Chain::Polygon => Some(NodeFixture {
            addresses: &["0x2A49C84B7173e21f9116B2798735f87531526b36"],
            transaction_hashes: &["0x3d4eb72380e6095d0667c6ec3420719dbec7d1d8b1628464a03ee6850ee716ed"],
        }),
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Solana
        | Chain::Thorchain
        | Chain::Mayachain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Arbitrum
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Zcash
        | Chain::Optimism
        | Chain::Aptos
        | Chain::Base
        | Chain::AvalancheC
        | Chain::Sui
        | Chain::Xrp
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei
        | Chain::SeiEvm
        | Chain::Manta
        | Chain::Blast
        | Chain::Noble
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo
        | Chain::Near
        | Chain::World
        | Chain::Stellar
        | Chain::Sonic
        | Chain::Algorand
        | Chain::Polkadot
        | Chain::Plasma
        | Chain::Cardano
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink
        | Chain::Unichain
        | Chain::Hyperliquid
        | Chain::HyperCore
        | Chain::Monad
        | Chain::XLayer
        | Chain::Robinhood
        | Chain::Stable => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_fixtures() {
        for chain in [Chain::Ethereum, Chain::SmartChain, Chain::Polygon] {
            let fixture = fixture(chain).unwrap();
            assert!(fixture.addresses.iter().all(|address| address.starts_with("0x") && address.len() == 42));
            assert!(fixture.transaction_hashes.iter().all(|hash| hash.starts_with("0x") && hash.len() == 66));
        }
        assert!(fixture(Chain::Base).is_none());
    }
}
