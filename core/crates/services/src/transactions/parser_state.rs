use std::error::Error;
use std::sync::Arc;

use primitives::Chain;
use storage::ParserState;

use super::repository::Repository;
use crate::Services;

pub struct ParserStateService {
    chain: Chain,
    repository: Arc<dyn Repository>,
}

impl ParserStateService {
    pub(crate) fn new(chain: Chain, repository: Arc<dyn Repository>) -> Self {
        Self { chain, repository }
    }

    pub async fn get_state(&self) -> Result<ParserState, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.get_parser_state(self.chain).await?)
    }

    pub async fn set_current_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.repository.set_parser_current_block(self.chain, block).await?;
        Ok(())
    }

    pub async fn set_latest_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.repository.set_parser_latest_block(self.chain, block).await?;
        Ok(())
    }
}

impl Services {
    pub fn parser_state(&self, chain: Chain) -> ParserStateService {
        ParserStateService::new(chain, self.transactions_repository())
    }

    pub async fn parser_chains(&self) -> Result<Vec<Chain>, Box<dyn Error + Send + Sync>> {
        Ok(self.transactions_repository().get_parser_states().await?.into_iter().map(|state| state.chain).collect())
    }
}
