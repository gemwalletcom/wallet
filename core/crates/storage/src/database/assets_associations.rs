use diesel::prelude::*;
use diesel::upsert::excluded;

use crate::DatabaseClient;
use crate::models::AssetAssociationRow;

pub(crate) trait AssetsAssociationsStore {
    fn upsert_asset_associations(&mut self, values: Vec<AssetAssociationRow>) -> Result<usize, diesel::result::Error>;
    fn get_asset_associations(&mut self, requested_asset_id: &str) -> Result<Vec<AssetAssociationRow>, diesel::result::Error>;
    fn get_assets_associations(&mut self) -> Result<Vec<AssetAssociationRow>, diesel::result::Error>;
}

impl AssetsAssociationsStore for DatabaseClient {
    fn upsert_asset_associations(&mut self, values: Vec<AssetAssociationRow>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets_associations::dsl::*;

        if values.is_empty() {
            return Ok(0);
        }

        diesel::insert_into(assets_associations)
            .values(values)
            .on_conflict((asset_id, associated_asset_id))
            .do_update()
            .set(association_type.eq(excluded(association_type)))
            .execute(&mut self.connection)
    }

    fn get_asset_associations(&mut self, requested_asset_id: &str) -> Result<Vec<AssetAssociationRow>, diesel::result::Error> {
        use crate::schema::{assets, assets_associations};

        assets_associations::table
            .inner_join(assets::table.on(assets::id.eq(assets_associations::associated_asset_id)))
            .filter(assets_associations::asset_id.eq(requested_asset_id))
            .order((assets::rank.desc(), assets_associations::associated_asset_id.asc()))
            .select(AssetAssociationRow::as_select())
            .load(&mut self.connection)
    }

    fn get_assets_associations(&mut self) -> Result<Vec<AssetAssociationRow>, diesel::result::Error> {
        use crate::schema::{assets, assets_associations};

        assets_associations::table
            .inner_join(assets::table.on(assets::id.eq(assets_associations::associated_asset_id)))
            .order((assets_associations::asset_id.asc(), assets::rank.desc(), assets_associations::associated_asset_id.asc()))
            .select(AssetAssociationRow::as_select())
            .load(&mut self.connection)
    }
}
