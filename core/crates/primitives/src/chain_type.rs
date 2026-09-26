use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ChainType {
    Ethereum,
    Bitcoin,
    Solana,
    Cosmos,
    Ton,
    Tron,
    Aptos,
    Sui,
    Xrp,
    Near,
    Stellar,
    Algorand,
    Polkadot,
    Cardano,
    HyperCore,
}

impl ChainType {
    pub fn network_fee_is_total_cost(&self) -> bool {
        match self {
            Self::Ethereum => true,
            Self::Bitcoin | Self::Solana | Self::Cosmos | Self::Ton | Self::Tron | Self::Aptos | Self::Sui | Self::Xrp | Self::Near | Self::Stellar | Self::Algorand | Self::Polkadot | Self::Cardano | Self::HyperCore => false,
        }
    }
}
