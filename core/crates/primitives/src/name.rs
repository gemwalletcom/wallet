use crate::chain::Chain;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Hashable")]
pub struct NameRecord {
    pub name: String,
    pub chain: Chain,
    pub address: String,
    pub provider: NameProvider,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AsRefStr, EnumString)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NameProvider {
    Ud,
    Ens,
    Sns,
    Ton,
    Spaceid,
    Did,
    Suins,
    Aptos,
    Injective,
    Icns,
    Lens,
    Basenames,
    Hyperliquid,
    AllDomains,
    Near,
}
