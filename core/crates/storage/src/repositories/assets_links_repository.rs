use diesel::{prelude::*, upsert::excluded};
use primitives::{AssetId, AssetLink as PrimitiveAssetLink};

use crate::models::AssetLinkRow;
use crate::{DatabaseClient, DatabaseError};

pub trait AssetsLinksRepository {
    fn add_assets_links(&mut self, asset_id: &AssetId, values: Vec<PrimitiveAssetLink>) -> Result<usize, DatabaseError>;
    fn get_asset_links(&mut self, asset_id: &AssetId) -> Result<Vec<PrimitiveAssetLink>, DatabaseError>;
}

impl AssetsLinksRepository for DatabaseClient {
    fn add_assets_links(&mut self, asset_id_value: &AssetId, values: Vec<PrimitiveAssetLink>) -> Result<usize, DatabaseError> {
        let rows = values.into_iter().filter_map(|x| AssetLinkRow::from_primitive(asset_id_value, x)).collect::<Vec<_>>();
        use crate::schema::assets_links::dsl::*;
        Ok(diesel::insert_into(assets_links)
            .values(rows)
            .on_conflict((asset_id, link_type))
            .do_update()
            .set((url.eq(excluded(url)),))
            .execute(&mut self.connection)?)
    }

    fn get_asset_links(&mut self, asset_id_value: &AssetId) -> Result<Vec<PrimitiveAssetLink>, DatabaseError> {
        use crate::schema::assets_links::dsl::*;
        Ok(assets_links
            .filter(asset_id.eq(asset_id_value.to_string()))
            .select(AssetLinkRow::as_select())
            .load(&mut self.connection)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect())
    }
}
