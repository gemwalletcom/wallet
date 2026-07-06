use std::fmt;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ListProviderName {
    Coingecko,
}

impl ListProviderName {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn id(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for ListProviderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}
