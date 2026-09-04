// https://api.etherscan.io/api?module=gastracker&action=gasoracle&apikey=YourApiKeyToken

use crate::client::GemstoneFeeData;
use gem_client::{ClientExt, ReqwestClient, Target, build_path_with_query};
use num_bigint::BigInt;
use primitives::{PriorityFeeValue, fee::FeePriority};
use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str;
use std::error::Error;

const ETHERSCAN_URL: &str = "https://api.etherscan.io";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EtherscanResult {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub last_block: u64,
    pub propose_gas_price: String,
    pub fast_gas_price: String,
    #[serde(rename = "suggestBaseFee")]
    pub suggest_base_fee: String,
    #[serde(rename = "gasUsedRatio")]
    pub gas_used_ratio: String,
}

impl EtherscanResult {
    /// Converts the raw Etherscan gas oracle data into the common `GemstoneFeeData` format.
    pub fn fee_data(&self) -> GemstoneFeeData {
        let base_fee: f64 = self.suggest_base_fee.parse().unwrap();
        let propose_fee: f64 = self.propose_gas_price.parse().unwrap();
        let fast_fee: f64 = self.fast_gas_price.parse().unwrap();

        let gas_used_ratio_str = self
            .gas_used_ratio
            .split(',')
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|val| format!("{:.1}%", val * 100.0));

        GemstoneFeeData {
            latest_block: self.last_block,
            suggest_base_fee: self.suggest_base_fee.clone(),
            gas_used_ratio: gas_used_ratio_str,
            priority_fees: vec![
                PriorityFeeValue {
                    priority: FeePriority::Normal,
                    value: BigInt::from((propose_fee - base_fee) as i64),
                },
                PriorityFeeValue {
                    priority: FeePriority::Fast,
                    value: BigInt::from((fast_fee - base_fee) as i64),
                },
            ],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub message: String,
    pub result: EtherscanResult,
}

#[derive(Clone, Debug)]
enum EtherscanTarget {
    GasOracle,
}

impl Target for EtherscanTarget {
    fn path(&self) -> String {
        match self {
            Self::GasOracle => build_path_with_query("/api", &[("module", "gastracker"), ("action", "gasoracle")]),
        }
    }
}

pub struct EtherscanClient {
    client: ReqwestClient,
    api_key: String,
}

impl EtherscanClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: ReqwestClient::new(ETHERSCAN_URL.to_string(), gem_client::reqwest_client()),
            api_key,
        }
    }

    pub async fn fetch_gas_oracle(&self) -> Result<EtherscanResponse, Box<dyn Error + Send + Sync>> {
        let response: EtherscanResponse = self.client.get(EtherscanTarget::GasOracle).query(&[("apikey", self.api_key.as_str())]).await?;
        if response.status != "1" {
            return Err(format!("Etherscan API error: {}", response.message).into());
        }
        Ok(response)
    }
}
