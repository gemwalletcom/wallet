use diesel::prelude::*;
use primitives::ScanVerdict;
use serde::{Deserialize, Serialize};

use crate::sql_types::{ChainRow, ScanProviderRow, ScanTypeRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_detections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ScanDetectionRow {
    pub id: i32,
    pub scan_type: ScanTypeRow,
    pub chain: Option<ChainRow>,
    pub target: String,
    pub provider: ScanProviderRow,
    pub reason: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

impl ScanDetectionRow {
    pub fn into_primitive(self) -> ScanVerdict {
        ScanVerdict {
            scan_type: self.scan_type.0,
            chain: self.chain.map(|chain| chain.0),
            target: self.target,
            provider: self.provider.0,
            reason: self.reason,
        }
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::scan_detections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewScanDetectionRow {
    pub scan_type: ScanTypeRow,
    pub chain: Option<ChainRow>,
    pub target: String,
    pub provider: ScanProviderRow,
    pub reason: Option<String>,
}

impl NewScanDetectionRow {
    pub fn from_primitive(verdict: ScanVerdict) -> Self {
        Self {
            scan_type: verdict.scan_type.into(),
            chain: verdict.chain.map(ChainRow::from),
            target: verdict.target,
            provider: verdict.provider.into(),
            reason: verdict.reason,
        }
    }
}
