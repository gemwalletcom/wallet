use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, NameProvider};

use crate::model::NameQuery;

#[async_trait]
pub trait NameResolver: Send + Sync {
    fn provider(&self) -> NameProvider;
    fn domains(&self) -> Vec<&'static str>;
    fn chains(&self) -> Vec<Chain>;
    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;
}
