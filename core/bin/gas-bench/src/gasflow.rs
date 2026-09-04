// https://api.gasflow.dev/predict

use gem_client::{ClientError, ClientExt, ReqwestClient, Target};
use num_bigint::BigInt;
use primitives::{PriorityFeeValue, fee::FeePriority};
use serde::Deserialize;

use crate::client::GemstoneFeeData;

const GASFLOW_URL: &str = "https://api.gasflow.dev";

#[derive(Debug, Deserialize)]
pub struct PredictedQuantiles {
    pub normal: f64,
    pub fast: f64,
}

#[derive(Debug, Deserialize)]
pub struct NetworkMetrics {
    pub gas_ratio_5: f64,
}

#[derive(Debug, Deserialize)]
pub struct GasflowResponse {
    pub current_block_number: u64,
    pub current_base_fee_gwei: f64,
    pub predicted_quantiles: PredictedQuantiles,
    pub network_metrics: NetworkMetrics,
}

impl GasflowResponse {
    /// Converts the raw Gasflow API data into the common `GemstoneFeeData` format.
    pub fn fee_data(&self) -> GemstoneFeeData {
        let gas_used_ratio_str = Some(format!("{:.1}%", self.network_metrics.gas_ratio_5 * 100.0));

        GemstoneFeeData {
            latest_block: self.current_block_number,
            suggest_base_fee: self.current_base_fee_gwei.to_string(),
            gas_used_ratio: gas_used_ratio_str,
            priority_fees: vec![
                PriorityFeeValue {
                    priority: FeePriority::Normal,
                    value: BigInt::from(self.predicted_quantiles.normal as i64),
                },
                PriorityFeeValue {
                    priority: FeePriority::Fast,
                    value: BigInt::from(self.predicted_quantiles.fast as i64),
                },
            ],
        }
    }
}

#[derive(Clone, Debug)]
enum GasflowTarget {
    Predict,
}

impl Target for GasflowTarget {
    fn path(&self) -> String {
        match self {
            Self::Predict => "/predict".to_string(),
        }
    }
}

pub struct GasflowClient {
    client: ReqwestClient,
}

impl Default for GasflowClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GasflowClient {
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::new(GASFLOW_URL.to_string(), gem_client::reqwest_client()),
        }
    }

    pub async fn fetch_prediction(&self) -> Result<GasflowResponse, ClientError> {
        self.client.get(GasflowTarget::Predict).await
    }
}
