use chrono::NaiveDateTime;
use diesel::Connection;
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::{AssetAddress as PrimitiveAssetAddress, AssetId, ChainAddress};

use crate::models::{AssetAddressRow, AssetAddressRowsExt};
use crate::{DatabaseClient, DatabaseError};

pub trait AssetsAddressesRepository {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError>;
    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_datetime: Option<NaiveDateTime>) -> Result<Vec<AssetId>, DatabaseError>;
    fn get_asset_addresses(&mut self, value: ChainAddress) -> Result<Vec<PrimitiveAssetAddress>, DatabaseError>;
    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError>;
}

impl AssetsAddressesRepository for DatabaseClient {
    fn add_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError> {
        use crate::schema::assets_addresses::dsl::*;
        use diesel::query_dsl::methods::FilterDsl;

        if values.is_empty() {
            return Ok(0);
        }
        let rows = values.into_iter().map(AssetAddressRow::from_primitive).collect::<Vec<_>>();
        let insert = diesel::insert_into(assets_addresses).values(&rows).on_conflict((asset_id, address)).do_update().set(value.eq(excluded(value)));

        Ok(insert.filter(excluded(value).is_not_null().and(value.is_distinct_from(excluded(value)))).execute(&mut self.connection)?)
    }

    fn get_assets_by_addresses(&mut self, values: Vec<ChainAddress>, from_datetime: Option<NaiveDateTime>) -> Result<Vec<AssetId>, DatabaseError> {
        let chains = values.iter().map(|x| x.chain.as_ref()).collect::<Vec<&str>>();
        let addresses = values.iter().map(|x| x.address.clone()).collect::<Vec<String>>();
        use crate::schema::{assets, assets_addresses::dsl as a};

        let mut query = a::assets_addresses
            .filter(a::chain.eq_any(chains))
            .filter(a::address.eq_any(addresses))
            .filter(a::value.is_null().or(a::value.ne("0")))
            .filter(diesel::dsl::exists(assets::table.filter(assets::id.eq(a::asset_id)).filter(assets::has_price.eq(true))))
            .select(AssetAddressRow::as_select())
            .into_boxed();

        if let Some(datetime) = from_datetime {
            query = query.filter(a::created_at.gt(datetime));
        }

        let rows: Vec<AssetAddressRow> = query.load(&mut self.connection)?;
        Ok(rows.asset_ids())
    }

    fn get_asset_addresses(&mut self, chain_address: ChainAddress) -> Result<Vec<PrimitiveAssetAddress>, DatabaseError> {
        use crate::schema::assets_addresses::dsl::*;
        assets_addresses
            .filter(chain.eq(chain_address.chain.as_ref()))
            .filter(address.eq(chain_address.address))
            .select(AssetAddressRow::as_select())
            .load(&mut self.connection)?
            .into_iter()
            .map(|row| row.as_primitive())
            .collect()
    }

    fn delete_assets_addresses(&mut self, values: Vec<PrimitiveAssetAddress>) -> Result<usize, DatabaseError> {
        use crate::schema::assets_addresses::dsl::*;

        if values.is_empty() {
            return Ok(0);
        }
        let rows = values.into_iter().map(AssetAddressRow::from_primitive).collect::<Vec<_>>();

        Ok(self.connection.transaction::<_, diesel::result::Error, _>(|connection| {
            let mut deleted = 0;

            for row in rows {
                deleted += diesel::delete(assets_addresses.filter(chain.eq(&row.chain)).filter(asset_id.eq(&row.asset_id)).filter(address.eq(&row.address))).execute(connection)?;
            }

            Ok(deleted)
        })?)
    }
}
