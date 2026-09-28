use diesel::{prelude::*, upsert::excluded};
use primitives::{PlatformStore, Release};

use crate::models::ReleaseRow;
use crate::sql_types::PlatformStore as PlatformStoreRow;
use crate::{DatabaseClient, DatabaseError};

pub trait ReleasesRepository {
    fn get_releases(&mut self) -> Result<Vec<Release>, DatabaseError>;
    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, DatabaseError>;
    fn update_release(&mut self, release: Release) -> Result<usize, DatabaseError>;
    fn is_update_enabled(&mut self, store: PlatformStore) -> Result<bool, DatabaseError>;
}

impl ReleasesRepository for DatabaseClient {
    fn get_releases(&mut self) -> Result<Vec<Release>, DatabaseError> {
        use crate::schema::releases::dsl::*;
        let rows = releases.order(updated_at.desc()).select(ReleaseRow::as_select()).load(&mut self.connection)?;
        Ok(rows.iter().map(ReleaseRow::as_primitive).collect())
    }

    fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, DatabaseError> {
        use crate::schema::releases::dsl::*;
        let values = values.into_iter().map(ReleaseRow::from_primitive).collect::<Vec<_>>();
        Ok(diesel::insert_into(releases).values(&values).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn update_release(&mut self, release: Release) -> Result<usize, DatabaseError> {
        use crate::schema::releases::dsl::*;
        let release = ReleaseRow::from_primitive(release);
        Ok(diesel::insert_into(releases)
            .values(&release)
            .on_conflict(platform_store)
            .do_update()
            .set(version.eq(excluded(version)))
            .execute(&mut self.connection)?)
    }

    fn is_update_enabled(&mut self, store: PlatformStore) -> Result<bool, DatabaseError> {
        let store: PlatformStoreRow = store.into();
        use crate::schema::releases::dsl::*;
        let release = releases.filter(platform_store.eq(&store)).select(ReleaseRow::as_select()).first(&mut self.connection).optional()?;
        Ok(release.map(|r| r.update_enabled).unwrap_or(true))
    }
}
