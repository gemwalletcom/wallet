use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumString, Eq, PartialOrd, Ord, Serialize, Deserialize, EnumIter)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    Aerodrome,
    Panora,
    Thorchain,
    Jupiter,
    Okx,
    Across,
    Oku,
    Wagmi,
    StonfiV2,
    Mayan,
    Chainflip,
    NearIntents,
    #[strum(to_string = "cetus_clmm", serialize = "cetus_aggregator")]
    #[serde(alias = "cetus_aggregator")]
    CetusClmm,
    Relay,
    Hyperliquid,
    Orca,
    Squid,
    Mayachain,
    SwapsXyz,
}

impl SwapProvider {
    pub fn id(&self) -> &str {
        self.as_ref()
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn is_cross_chain(&self) -> bool {
        match self {
            Self::Thorchain | Self::Mayachain | Self::Across | Self::Mayan | Self::Chainflip | Self::NearIntents | Self::Relay | Self::Hyperliquid | Self::Squid | Self::SwapsXyz => true,
            Self::UniswapV3 | Self::UniswapV4 | Self::PancakeswapV3 | Self::Panora | Self::Jupiter | Self::Okx | Self::Oku | Self::Wagmi | Self::CetusClmm | Self::StonfiV2 | Self::Aerodrome | Self::Orca => false,
        }
    }

    pub fn cross_chain_providers() -> Vec<Self> {
        Self::all().into_iter().filter(Self::is_cross_chain).collect()
    }

    pub fn name(&self) -> &str {
        match self {
            Self::UniswapV3 | Self::UniswapV4 => "Uniswap",
            Self::PancakeswapV3 => "PancakeSwap",
            Self::Aerodrome => "Aerodrome",
            Self::Panora => "Panora",
            Self::Thorchain => "THORChain",
            Self::Mayachain => "Maya",
            Self::Jupiter => "Jupiter",
            Self::Okx => "OKX (DEX)",
            Self::Across => "Across",
            Self::Oku => "Oku",
            Self::Wagmi => "Wagmi",
            Self::CetusClmm => "Cetus",
            Self::StonfiV2 => "STON.fi",
            Self::Mayan => "Mayan",
            Self::Chainflip => "Chainflip",
            Self::NearIntents => "NEAR Intents",
            Self::Relay => "Relay",
            Self::Hyperliquid => "Hyperliquid",
            Self::Orca => "Orca",
            Self::Squid => "Squid",
            Self::SwapsXyz => "Swaps.xyz",
        }
    }

    pub fn protocol_name(&self) -> &str {
        match self {
            Self::UniswapV3 => "Uniswap v3",
            Self::UniswapV4 => "Uniswap v4",
            Self::PancakeswapV3 => "PancakeSwap v3",
            Self::Panora => "Panora",
            Self::Across => "Across v3",
            Self::Oku => "Oku",
            Self::StonfiV2 => "STON.fi v2",
            Self::CetusClmm => "Cetus",
            Self::Thorchain | Self::Mayachain | Self::Jupiter | Self::Okx | Self::Wagmi | Self::Mayan | Self::Chainflip | Self::NearIntents | Self::Aerodrome | Self::Relay | Self::Hyperliquid | Self::Orca | Self::Squid | Self::SwapsXyz => {
                self.name()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_is_cross_chain() {
        assert!(SwapProvider::Thorchain.is_cross_chain());
        assert!(SwapProvider::Across.is_cross_chain());
        assert!(SwapProvider::Mayan.is_cross_chain());
        assert!(SwapProvider::NearIntents.is_cross_chain());
        assert!(SwapProvider::Relay.is_cross_chain());
        assert!(SwapProvider::SwapsXyz.is_cross_chain());
        assert!(!SwapProvider::UniswapV3.is_cross_chain());
        assert!(!SwapProvider::Jupiter.is_cross_chain());
    }

    #[test]
    fn test_a_swap_stored_under_the_old_cetus_id_still_reads_back() {
        assert_eq!(SwapProvider::from_str("cetus_aggregator").unwrap(), SwapProvider::CetusClmm);
        assert_eq!(SwapProvider::from_str("cetus_clmm").unwrap(), SwapProvider::CetusClmm);
        assert_eq!(SwapProvider::CetusClmm.id(), "cetus_clmm");
        assert_eq!(SwapProvider::CetusClmm.name(), "Cetus");
        assert_eq!(
            serde_json::from_str::<SwapProvider>("\"cetus_aggregator\"").unwrap(),
            SwapProvider::CetusClmm,
            "a stored swap row written before the rename still decodes"
        );
        assert_eq!(serde_json::to_string(&SwapProvider::CetusClmm).unwrap(), "\"cetus_clmm\"");
    }
}
