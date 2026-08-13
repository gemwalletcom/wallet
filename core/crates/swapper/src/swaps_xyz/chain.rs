use primitives::{Asset, Chain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SwapsXyzChain {
    pub(super) chain: Chain,
    pub(super) id: u64,
    pub(super) key: &'static str,
}

impl SwapsXyzChain {
    pub(super) const ALL: &'static [Self] = &[
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
            chain: Chain::Zcash,
            id: 999_000_322,
            key: "zec",
        },
        Self {
            chain: Chain::Near,
            id: 397,
            key: "near",
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
    ];

    pub(super) fn from_chain(chain: Chain) -> Option<Self> {
        Self::ALL.iter().copied().find(|value| value.chain == chain)
    }

    pub(super) fn from_id(id: u64) -> Option<Self> {
        Self::ALL.iter().copied().find(|value| value.id == id)
    }

    pub(super) fn decimals(self) -> u32 {
        Asset::from_chain(self.chain).decimals as u32
    }
}
