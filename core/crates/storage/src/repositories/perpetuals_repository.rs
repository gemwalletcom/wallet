use diesel::{prelude::*, upsert::excluded};
use primitives::{AssetId, perpetual::Perpetual};

use crate::models::{NewPerpetualRow, PerpetualRow};
use crate::schema::{perpetuals, perpetuals_assets};
use crate::{DatabaseClient, DatabaseError};

pub trait PerpetualsRepository {
    fn get_perpetuals_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<Perpetual>, DatabaseError>;

    fn perpetuals_update(&mut self, values: Vec<Perpetual>) -> Result<usize, DatabaseError>;

    fn get_perpetuals(&mut self) -> Result<Vec<Perpetual>, DatabaseError>;
}

impl PerpetualsRepository for DatabaseClient {
    fn get_perpetuals_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<Perpetual>, DatabaseError> {
        Ok(perpetuals::table
            .inner_join(perpetuals_assets::table.on(perpetuals::id.eq(perpetuals_assets::perpetual_id)))
            .filter(perpetuals_assets::asset_id.eq(asset_id.to_string()))
            .select(PerpetualRow::as_select())
            .load(&mut self.connection)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }

    fn perpetuals_update(&mut self, values: Vec<Perpetual>) -> Result<usize, DatabaseError> {
        if values.is_empty() {
            return Ok(0);
        }
        let values = values.into_iter().map(NewPerpetualRow::from_primitive).collect::<Vec<_>>();
        Ok(diesel::insert_into(perpetuals::table)
            .values(&values)
            .on_conflict(perpetuals::id)
            .do_update()
            .set((
                perpetuals::name.eq(excluded(perpetuals::name)),
                perpetuals::provider.eq(excluded(perpetuals::provider)),
                perpetuals::asset_id.eq(excluded(perpetuals::asset_id)),
                perpetuals::price.eq(excluded(perpetuals::price)),
                perpetuals::price_percent_change_24h.eq(excluded(perpetuals::price_percent_change_24h)),
                perpetuals::open_interest.eq(excluded(perpetuals::open_interest)),
                perpetuals::volume_24h.eq(excluded(perpetuals::volume_24h)),
                perpetuals::funding.eq(excluded(perpetuals::funding)),
                perpetuals::leverage.eq(excluded(perpetuals::leverage)),
            ))
            .execute(&mut self.connection)?)
    }

    fn get_perpetuals(&mut self) -> Result<Vec<Perpetual>, DatabaseError> {
        let rows = perpetuals::table.select(PerpetualRow::as_select()).load(&mut self.connection)?;
        Ok(rows.iter().map(PerpetualRow::as_primitive).collect())
    }
}
