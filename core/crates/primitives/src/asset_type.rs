use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AssetType {
    NATIVE,
    ERC20,
    BEP20,
    SPL,
    SPL2022,
    TRC20,
    TIP20,
    TOKEN,
    IBC,
    JETTON,
    SYNTH,
    ASA,
    PERPETUAL,
    SPOT,
}

impl AssetType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetSubtype {
    NATIVE,
    TOKEN,
}
