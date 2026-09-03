pub mod client;
pub mod factory;
pub mod model;
pub mod providers;
pub mod resolver;
#[cfg(test)]
pub mod testkit;

pub use client::{NameClient, NameConfig};
pub use factory::NameProviderFactory;
pub use model::NameQuery;
pub use resolver::NameResolver;
