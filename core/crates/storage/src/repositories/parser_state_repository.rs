use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::Chain;

use crate::models::ParserStateRow;
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserState {
    pub chain: Chain,
    pub current_block: i64,
    pub latest_block: i64,
    pub await_blocks: i32,
    pub timeout_between_blocks: i32,
    pub timeout_latest_block: i32,
    pub parallel_blocks: i32,
    pub is_enabled: bool,
    pub updated_at: NaiveDateTime,
    pub queue_behind_blocks: Option<i32>,
    pub block_time: i32,
}

impl ParserState {
    fn from_row(row: ParserStateRow) -> Self {
        Self {
            chain: row.chain.0,
            current_block: row.current_block,
            latest_block: row.latest_block,
            await_blocks: row.await_blocks,
            timeout_between_blocks: row.timeout_between_blocks,
            timeout_latest_block: row.timeout_latest_block,
            parallel_blocks: row.parallel_blocks,
            is_enabled: row.is_enabled,
            updated_at: row.updated_at,
            queue_behind_blocks: row.queue_behind_blocks,
            block_time: row.block_time,
        }
    }
}

pub trait ParserStateRepository {
    fn get_parser_state(&mut self, chain: Chain) -> Result<ParserState, DatabaseError>;
    fn add_parser_state(&mut self, chain: Chain, block_time_ms: i32) -> Result<usize, DatabaseError>;
    fn get_parser_states(&mut self) -> Result<Vec<ParserState>, DatabaseError>;
    fn set_parser_state_latest_block(&mut self, chain: Chain, block: i64) -> Result<usize, DatabaseError>;
    fn set_parser_state_current_block(&mut self, chain: Chain, block: i64) -> Result<usize, DatabaseError>;
}

impl ParserStateRepository for DatabaseClient {
    fn get_parser_state(&mut self, chain_value: Chain) -> Result<ParserState, DatabaseError> {
        use crate::schema::parser_state::dsl::*;
        let row = parser_state
            .filter(chain.eq(ChainRow::from(chain_value)))
            .select(ParserStateRow::as_select())
            .first(&mut self.connection)
            .or_not_found(chain_value.as_ref().to_string())?;
        Ok(ParserState::from_row(row))
    }

    fn add_parser_state(&mut self, chain_value: Chain, block_time_ms: i32) -> Result<usize, DatabaseError> {
        use crate::schema::parser_state::dsl::*;
        Ok(diesel::insert_into(parser_state)
            .values((chain.eq(ChainRow::from(chain_value)), block_time.eq(block_time_ms)))
            .on_conflict_do_nothing()
            .execute(&mut self.connection)?)
    }

    fn get_parser_states(&mut self) -> Result<Vec<ParserState>, DatabaseError> {
        use crate::schema::parser_state::dsl::*;
        let rows = parser_state.select(ParserStateRow::as_select()).load(&mut self.connection)?;
        Ok(rows.into_iter().map(ParserState::from_row).collect())
    }

    fn set_parser_state_latest_block(&mut self, chain_value: Chain, block: i64) -> Result<usize, DatabaseError> {
        use crate::schema::parser_state::dsl::*;
        Ok(diesel::update(parser_state.find(ChainRow::from(chain_value))).set(latest_block.eq(block)).execute(&mut self.connection)?)
    }

    fn set_parser_state_current_block(&mut self, chain_value: Chain, block: i64) -> Result<usize, DatabaseError> {
        use crate::schema::parser_state::dsl::*;
        Ok(diesel::update(parser_state.find(ChainRow::from(chain_value))).set(current_block.eq(block)).execute(&mut self.connection)?)
    }
}
