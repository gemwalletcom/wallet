use primitives::{AssetId, ChartPeriod};

use crate::method::GemApiMethod;

#[derive(Clone, Debug)]
pub enum GemApiTarget {
    GetCharts(AssetId, ChartPeriod),
}

impl GemApiTarget {
    pub fn method(&self) -> GemApiMethod {
        match self {
            Self::GetCharts(_, _) => GemApiMethod::Get,
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::GetCharts(asset_id, period) => format!("/v1/charts/{asset_id}?period={}", period.as_ref()),
        }
    }
}
