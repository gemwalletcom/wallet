pub mod client;
mod coalescing_provider;
pub mod error;
pub mod provider;
#[cfg(feature = "reqwest_provider")]
pub mod reqwest_provider;
pub mod target;

use std::sync::Arc;

pub use client::{AlienClient, new_alien_client};
pub use error::AlienError;
pub use provider::{AlienProvider, AlienProviderWrapper};
pub use target::{AlienHttpMethod, AlienResponse, AlienTarget, X_CACHE_TTL};

pub(crate) fn coalescing_provider(provider: Arc<dyn AlienProvider>) -> Arc<dyn AlienProvider> {
    Arc::new(coalescing_provider::CoalescingAlienProvider::new(provider))
}
