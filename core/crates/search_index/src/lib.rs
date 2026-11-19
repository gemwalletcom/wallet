mod meilisearch;
pub mod models;
mod search;

pub use meilisearch::*;
pub use models::*;
pub use search::{SearchProvider, SearchQuery};

#[cfg(any(test, debug_assertions))]
pub mod testkit;
