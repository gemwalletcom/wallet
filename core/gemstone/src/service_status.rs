use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use primitives::{GEM_API_HOST, GEM_NODES_ASIA_HOST, GEM_NODES_EUROPE_HOST, GEM_NODES_HOST};

use crate::GemstoneError;
use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};

const FLAG_UNITED_STATES: &str = "🇺🇸";
const FLAG_JAPAN: &str = "🇯🇵";
const FLAG_EUROPE: &str = "🇪🇺";

const TIMEOUT_SECONDS: u32 = 10;

#[uniffi::export]
pub fn service_status_timeout_seconds() -> u32 {
    TIMEOUT_SECONDS
}

#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum GemServiceEndpointType {
    Api,
    GemNode,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemServiceEndpoint {
    pub endpoint_type: GemServiceEndpointType,
    pub host: String,
    pub url: String,
    pub flag: String,
}

impl GemServiceEndpoint {
    fn new(endpoint_type: GemServiceEndpointType, host: &str, flag: &str) -> Self {
        Self {
            endpoint_type,
            host: host.to_string(),
            url: format!("https://{host}"),
            flag: flag.to_string(),
        }
    }
}

#[derive(uniffi::Object)]
pub struct GemServiceStatus {
    provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl GemServiceStatus {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub fn get_endpoints(&self) -> Vec<GemServiceEndpoint> {
        vec![
            GemServiceEndpoint::new(GemServiceEndpointType::Api, GEM_API_HOST, FLAG_UNITED_STATES),
            GemServiceEndpoint::new(GemServiceEndpointType::GemNode, GEM_NODES_HOST, FLAG_UNITED_STATES),
            GemServiceEndpoint::new(GemServiceEndpointType::GemNode, GEM_NODES_ASIA_HOST, FLAG_JAPAN),
            GemServiceEndpoint::new(GemServiceEndpointType::GemNode, GEM_NODES_EUROPE_HOST, FLAG_EUROPE),
        ]
    }

    pub async fn get_endpoint_latency(&self, url: String) -> Result<u64, GemstoneError> {
        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: Some(HashMap::from([("Cache-Control".to_string(), "no-cache".to_string())])),
            body: None,
        };
        let start_time = Instant::now();
        self.provider.request(target).await?;

        Ok(start_time.elapsed().as_millis() as u64)
    }
}
