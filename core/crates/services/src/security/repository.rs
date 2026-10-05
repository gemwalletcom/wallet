use std::time::Duration;

use async_trait::async_trait;
use primitives::{AssetBasic, AssetId, Chain, ScanAddress, ScanVerdict};
use storage::{AssetsRepository, Database, DatabaseError, ScanAddressesRepository, ScanDetectionsRepository};

pub(crate) struct ScanRecords {
    pub(crate) addresses: Vec<ScanAddress>,
    pub(crate) assets: Vec<AssetBasic>,
    pub(crate) verdicts: Vec<ScanVerdict>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn scan_records(&self, addresses: Vec<(Chain, String)>, asset_ids: Vec<AssetId>, targets: Vec<String>, detection_max_age: Option<Duration>) -> Result<ScanRecords, DatabaseError>;
    async fn add_scan_detections(&self, verdicts: Vec<ScanVerdict>) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn scan_records(&self, addresses: Vec<(Chain, String)>, asset_ids: Vec<AssetId>, targets: Vec<String>, detection_max_age: Option<Duration>) -> Result<ScanRecords, DatabaseError> {
        self.database
            .run(move |client| {
                let queries = addresses.iter().map(|(chain, address)| (*chain, address.as_str())).collect::<Vec<_>>();
                Ok(ScanRecords {
                    addresses: client.get_scan_addresses(&queries)?,
                    assets: client.get_assets_basic(asset_ids)?,
                    verdicts: match detection_max_age {
                        Some(max_age) => client.get_scan_detections(targets, max_age)?,
                        None => Vec::new(),
                    },
                })
            })
            .await
    }

    async fn add_scan_detections(&self, verdicts: Vec<ScanVerdict>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_scan_detections(verdicts)).await
    }
}
