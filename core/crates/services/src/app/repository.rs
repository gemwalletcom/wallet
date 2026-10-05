use async_trait::async_trait;
use primitives::{AssetBasic, FeaturePolicy, Release};
use storage::{AssetFilter, AssetsRepository, Database, DatabaseError, FeaturesRepository, ReleasesRepository};

pub(crate) struct ConfigRecords {
    pub(crate) fiat_on_ramp_assets: Vec<AssetBasic>,
    pub(crate) fiat_off_ramp_assets: Vec<AssetBasic>,
    pub(crate) swap_assets: Vec<String>,
    pub(crate) releases: Vec<Release>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn config_records(&self, on_ramp_filters: Vec<AssetFilter>, off_ramp_filters: Vec<AssetFilter>) -> Result<ConfigRecords, DatabaseError>;
    async fn features(&self) -> Result<Vec<FeaturePolicy>, DatabaseError>;
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
    async fn config_records(&self, on_ramp_filters: Vec<AssetFilter>, off_ramp_filters: Vec<AssetFilter>) -> Result<ConfigRecords, DatabaseError> {
        self.database
            .run(move |client| {
                Ok(ConfigRecords {
                    fiat_on_ramp_assets: client.get_assets_by_filter(on_ramp_filters)?,
                    fiat_off_ramp_assets: client.get_assets_by_filter(off_ramp_filters)?,
                    swap_assets: client.get_swap_assets()?,
                    releases: client.get_releases()?,
                })
            })
            .await
    }

    async fn features(&self) -> Result<Vec<FeaturePolicy>, DatabaseError> {
        self.database.run(FeaturesRepository::get_features).await
    }
}
