use std::error::Error;

use primitives::Chain;
use storage::{Database, ParserState, ParserStateRepository};

use crate::Services;

pub struct ParserStateService {
    chain: Chain,
    database: Database,
}

impl ParserStateService {
    pub fn new(chain: Chain, database: Database) -> Self {
        Self { chain, database }
    }

    pub async fn get_state(&self) -> Result<ParserState, Box<dyn Error + Send + Sync>> {
        let chain = self.chain;
        Ok(self.database.run(move |client| client.get_parser_state(chain)).await?)
    }

    pub async fn set_current_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chain = self.chain;
        self.database.run(move |client| client.set_parser_state_current_block(chain, block)).await?;
        Ok(())
    }

    pub async fn set_latest_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let chain = self.chain;
        self.database.run(move |client| client.set_parser_state_latest_block(chain, block)).await?;
        Ok(())
    }
}

impl Services {
    pub fn parser_state(&self, chain: Chain) -> ParserStateService {
        ParserStateService::new(chain, self.database())
    }

    pub async fn parser_chains(&self) -> Result<Vec<Chain>, Box<dyn Error + Send + Sync>> {
        Ok(self.database().run(ParserStateRepository::get_parser_states).await?.into_iter().map(|state| state.chain).collect())
    }
}
