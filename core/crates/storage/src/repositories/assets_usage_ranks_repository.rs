use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::AssetId;

use crate::models::AssetUsageRankRow;
use crate::{DatabaseClient, DatabaseError};

pub trait AssetsUsageRanksRepository {
    fn upsert_usage_ranks(&mut self, values: &[(AssetId, i32)]) -> Result<usize, DatabaseError>;
    fn delete_usage_ranks_before(&mut self, before: NaiveDateTime) -> Result<usize, DatabaseError>;
    fn get_all_usage_ranks(&mut self) -> Result<Vec<(AssetId, i32)>, DatabaseError>;
}

impl AssetsUsageRanksRepository for DatabaseClient {
    fn upsert_usage_ranks(&mut self, values: &[(AssetId, i32)]) -> Result<usize, DatabaseError> {
        use crate::schema::assets_usage_ranks::dsl::*;
        if values.is_empty() {
            return Ok(0);
        }
        let rows = values
            .iter()
            .map(|(value_asset_id, value_usage_rank)| AssetUsageRankRow {
                asset_id: value_asset_id.clone().into(),
                usage_rank: *value_usage_rank,
            })
            .collect::<Vec<_>>();
        Ok(diesel::insert_into(assets_usage_ranks)
            .values(&rows)
            .on_conflict(asset_id)
            .do_update()
            .set(usage_rank.eq(excluded(usage_rank)))
            .execute(&mut self.connection)?)
    }

    fn delete_usage_ranks_before(&mut self, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        use crate::schema::assets_usage_ranks::dsl::*;
        Ok(diesel::delete(assets_usage_ranks.filter(updated_at.lt(before))).execute(&mut self.connection)?)
    }

    fn get_all_usage_ranks(&mut self) -> Result<Vec<(AssetId, i32)>, DatabaseError> {
        use crate::schema::assets_usage_ranks::dsl::*;
        let rows = assets_usage_ranks.select(AssetUsageRankRow::as_select()).load(&mut self.connection)?;
        Ok(rows.into_iter().map(|row| (row.asset_id.0, row.usage_rank)).collect())
    }
}
