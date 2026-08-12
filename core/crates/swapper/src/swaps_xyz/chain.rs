use primitives::{Asset, Chain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SwapsXyzChain {
    chain: Chain,
    id: u64,
    key: &'static str,
}

impl SwapsXyzChain {
    const ALL: &'static [Self] = &[
        Self {
            chain: Chain::Algorand,
            id: 999_000_419,
            key: "algorand",
        },
        Self {
            chain: Chain::Stellar,
            id: 999_000_338,
            key: "xlm",
        },
        Self {
            chain: Chain::Cardano,
            id: 1816,
            key: "cardano",
        },
        Self {
            chain: Chain::Ton,
            id: 999_000_337,
            key: "ton",
        },
        Self {
            chain: Chain::Cosmos,
            id: 999_000_433,
            key: "atom",
        },
        Self {
            chain: Chain::Osmosis,
            id: 999_000_446,
            key: "osmo",
        },
        Self {
            chain: Chain::Aptos,
            id: 999_000_325,
            key: "aptos",
        },
        Self {
            chain: Chain::Sui,
            id: 999_000_938,
            key: "sui",
        },
        Self {
            chain: Chain::Xrp,
            id: 999_000_346,
            key: "xrp",
        },
        Self {
            chain: Chain::Tron,
            id: 728_126_428,
            key: "trx",
        },
    ];

    pub(super) fn all() -> &'static [Self] {
        Self::ALL
    }

    pub(super) fn from_chain(chain: Chain) -> Option<Self> {
        Self::ALL.iter().copied().find(|value| value.chain == chain)
    }

    pub(super) fn from_id(id: u64) -> Option<Self> {
        Self::ALL.iter().copied().find(|value| value.id == id)
    }

    pub(super) fn chain(self) -> Chain {
        self.chain
    }

    pub(super) fn id(self) -> u64 {
        self.id
    }

    pub(super) fn key(self) -> &'static str {
        self.key
    }

    pub(super) fn decimals(self) -> u32 {
        Asset::from_chain(self.chain).decimals as u32
    }
}
