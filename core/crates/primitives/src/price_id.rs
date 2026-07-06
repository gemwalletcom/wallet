use std::fmt;
use std::str::FromStr;

use crate::{PriceProvider, provider_scoped_id::ProviderScopedId};

/// The resolved (provider, provider_price_id) pair used to key prices, charts and any other
/// provider-scoped data. `id()` produces the synthetic `prices.id` used across the schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceId {
    pub provider: PriceProvider,
    pub provider_price_id: String,
}

impl PriceId {
    pub fn new(provider: PriceProvider, provider_price_id: String) -> Self {
        Self { provider, provider_price_id }
    }

    pub fn id(&self) -> String {
        self.to_string()
    }

    pub fn id_for(provider: PriceProvider, provider_price_id: &str) -> String {
        ProviderScopedId::id_for(provider, provider_price_id)
    }
}

impl fmt::Display for PriceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&Self::id_for(self.provider, &self.provider_price_id))
    }
}

impl FromStr for PriceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = ProviderScopedId::parse(s)?;
        let provider = id.provider_id.parse().map_err(|_| format!("Unknown provider: {}", id.provider_id))?;
        Ok(Self {
            provider,
            provider_price_id: id.resource_id,
        })
    }
}

crate::impl_string_serde!(PriceId);
