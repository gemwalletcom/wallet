use gem_client::{Client, ClientError, ClientExt};
use primitives::{AssetId, ChartPeriod, Charts};

use crate::target::GemApiTarget;

#[derive(Debug, Clone)]
pub struct GemApiClient<C: Client> {
    client: C,
}

impl<C: Client> GemApiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_charts(&self, asset_id: AssetId, period: ChartPeriod) -> Result<Charts, ClientError> {
        self.client.get(&GemApiTarget::GetCharts(asset_id, period).path()).await
    }
}
