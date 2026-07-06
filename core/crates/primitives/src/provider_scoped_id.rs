use std::fmt;

use crate::CHAIN_SEPARATOR;

pub(crate) struct ProviderScopedId {
    pub(crate) provider_id: String,
    pub(crate) resource_id: String,
}

impl ProviderScopedId {
    pub(crate) fn id_for(provider_id: impl fmt::Display, resource_id: &str) -> String {
        format!("{provider_id}{CHAIN_SEPARATOR}{resource_id}")
    }

    pub(crate) fn parse(id: &str) -> Result<Self, String> {
        let (provider_id, resource_id) = id.split_once(CHAIN_SEPARATOR).ok_or_else(|| format!("Invalid provider scoped id: {id}"))?;
        if provider_id.is_empty() || resource_id.is_empty() {
            return Err(format!("Invalid provider scoped id: {id}"));
        }
        Ok(Self {
            provider_id: provider_id.to_string(),
            resource_id: resource_id.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_scoped_id_formats_provider_and_resource() {
        assert_eq!(ProviderScopedId::id_for("coingecko", "xstocks-ecosystem"), "coingecko_xstocks-ecosystem");
    }

    #[test]
    fn test_provider_scoped_id_parses_provider_and_resource() {
        let id = ProviderScopedId::parse("coingecko_xstocks-ecosystem").unwrap();

        assert_eq!(id.provider_id, "coingecko");
        assert_eq!(id.resource_id, "xstocks-ecosystem");
    }

    #[test]
    fn test_provider_scoped_id_rejects_invalid_ids() {
        assert!(ProviderScopedId::parse("xstocks-ecosystem").is_err());
        assert!(ProviderScopedId::parse("_xstocks-ecosystem").is_err());
        assert!(ProviderScopedId::parse("coingecko_").is_err());
    }
}
