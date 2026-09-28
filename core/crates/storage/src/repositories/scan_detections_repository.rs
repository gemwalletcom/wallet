use std::time::Duration;

use chrono::Utc;
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::ScanVerdict;

use crate::models::{NewScanDetectionRow, ScanDetectionRow};
use crate::{DatabaseClient, DatabaseError};

pub trait ScanDetectionsRepository {
    fn get_scan_detections(&mut self, targets: Vec<String>, max_age: Duration) -> Result<Vec<ScanVerdict>, DatabaseError>;
    fn add_scan_detections(&mut self, values: Vec<ScanVerdict>) -> Result<usize, DatabaseError>;
}

impl ScanDetectionsRepository for DatabaseClient {
    fn get_scan_detections(&mut self, targets: Vec<String>, max_age: Duration) -> Result<Vec<ScanVerdict>, DatabaseError> {
        let since = Utc::now().naive_utc() - max_age;
        use crate::schema::scan_detections::dsl::*;
        Ok(scan_detections
            .filter(target.eq_any(targets))
            .filter(updated_at.ge(since))
            .select(ScanDetectionRow::as_select())
            .load(&mut self.connection)?
            .into_iter()
            .map(ScanDetectionRow::into_primitive)
            .collect())
    }

    fn add_scan_detections(&mut self, values: Vec<ScanVerdict>) -> Result<usize, DatabaseError> {
        use crate::schema::scan_detections::dsl::*;
        if values.is_empty() {
            return Ok(0);
        }
        Ok(diesel::insert_into(scan_detections)
            .values(values.into_iter().map(NewScanDetectionRow::from_primitive).collect::<Vec<_>>())
            .on_conflict((scan_type, chain, target))
            .do_update()
            .set((provider.eq(excluded(provider)), reason.eq(excluded(reason)), updated_at.eq(diesel::dsl::now)))
            .execute(&mut self.connection)?)
    }
}
