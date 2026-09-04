use super::{HeliusPriorityFeeOptions, HeliusPriorityFeeParams, HeliusPriorityFeeRequest, HeliusPriorityFeeResponse, HeliusPriorityFees};
use gem_client::{ClientExt, ReqwestClient, Target};
use std::error::Error;

const HELIUS_URL: &str = "https://mainnet.helius-rpc.com";

#[derive(Clone, Debug)]
enum HeliusTarget {
    Rpc,
}

impl Target for HeliusTarget {
    fn path(&self) -> String {
        match self {
            Self::Rpc => "/".to_string(),
        }
    }
}

pub struct HeliusClient {
    client: ReqwestClient,
    api_key: String,
}

impl HeliusClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: ReqwestClient::new(HELIUS_URL.to_string(), gem_client::reqwest_client()),
            api_key: api_key.to_string(),
        }
    }

    pub async fn fetch_priority_fee_estimate(&self, account_keys: Option<Vec<String>>) -> Result<HeliusPriorityFees, Box<dyn Error + Send + Sync>> {
        let request = HeliusPriorityFeeRequest {
            jsonrpc: "2.0",
            id: "1",
            method: "getPriorityFeeEstimate",
            params: vec![HeliusPriorityFeeParams {
                account_keys,
                options: HeliusPriorityFeeOptions {
                    include_all_priority_fee_levels: true,
                    lookback_slots: 150,
                },
            }],
        };

        let result: HeliusPriorityFeeResponse = self.client.post(HeliusTarget::Rpc, &request).query(&[("api-key", self.api_key.as_str())]).await?;

        let levels = result.result.priority_fee_levels.ok_or("No priority fee levels in response")?;

        Ok(HeliusPriorityFees::from_levels(&levels))
    }
}
