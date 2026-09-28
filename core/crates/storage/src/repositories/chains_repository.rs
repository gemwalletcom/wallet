use diesel::prelude::*;
use primitives::Chain;

use crate::models::ChainIdRow;
use crate::{DatabaseClient, DatabaseError};

pub trait ChainsRepository {
    fn add_chains(&mut self, chains: Vec<Chain>) -> Result<usize, DatabaseError>;
}

impl ChainsRepository for DatabaseClient {
    fn add_chains(&mut self, values: Vec<Chain>) -> Result<usize, DatabaseError> {
        let chain_values = values.into_iter().map(|chain_id| ChainIdRow { id: chain_id.into() }).collect::<Vec<_>>();
        use crate::schema::chains::dsl::*;
        Ok(diesel::insert_into(chains).values(chain_values).on_conflict_do_nothing().execute(&mut self.connection)?)
    }
}
