use std::fmt;
use std::str::FromStr;

use crate::{ListProviderName, provider_scoped_id::ProviderScopedId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListId {
    pub provider: ListProviderName,
    pub provider_list_id: String,
}

impl ListId {
    pub fn id(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for ListId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&ProviderScopedId::id_for(self.provider, &self.provider_list_id))
    }
}

impl FromStr for ListId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = ProviderScopedId::parse(s)?;
        let provider = id.provider_id.parse().map_err(|_| format!("Unknown provider: {}", id.provider_id))?;
        Ok(Self {
            provider,
            provider_list_id: id.resource_id,
        })
    }
}

crate::impl_string_serde!(ListId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_id_parses_provider_and_source() {
        let list_id = ListId::from_str("coingecko_xstocks-ecosystem").unwrap();

        assert_eq!(list_id.id(), "coingecko_xstocks-ecosystem");
        assert_eq!(list_id.provider, ListProviderName::Coingecko);
        assert_eq!(list_id.provider_list_id, "xstocks-ecosystem");
    }

    #[test]
    fn test_list_id_rejects_invalid_ids() {
        assert!(ListId::from_str("xstocks-ecosystem").is_err());
        assert!(ListId::from_str("_xstocks-ecosystem").is_err());
        assert!(ListId::from_str("coingecko_").is_err());
        assert!(ListId::from_str("unknown_xstocks-ecosystem").is_err());
    }
}
