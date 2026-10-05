mod meilisearch;
pub mod models;
mod search;

pub use meilisearch::*;
pub use models::*;
pub use search::{SearchProvider, SearchQuery};

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;
