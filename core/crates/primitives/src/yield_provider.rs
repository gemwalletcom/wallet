use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Display, AsRefStr, EnumString, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum YieldProvider {
    Yo,
}

impl YieldProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::Yo => "Yo",
        }
    }
}
