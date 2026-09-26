use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum StakeProviderType {
    Stake,
    Earn,
}
