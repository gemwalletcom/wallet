use std::{str::FromStr, time::Instant};

use chain_traits::NoopNodeCheckReporter;
use primitives::{Chain, NodeCheckProfile, NodeStatusState, NodeType};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use settings_chain::{ProviderConfig, ProviderFactory, ProviderKeyConfig};

use super::switch_reason::CurrentNodeErrorKind;
use super::sync::NodeStatusObservation;
use crate::config::Url;

pub struct ChainClient {
    config: ProviderConfig,
    profile: Option<NodeCheckProfile>,
    url: Url,
}

impl ChainClient {
    pub fn new(chain: Chain, profile: Option<NodeCheckProfile>, url: Url) -> Self {
        let config = ProviderConfig::new(chain, &url.url, NodeType::Default, ProviderKeyConfig::default());
        Self { config, profile, url }
    }

    pub async fn get_status(&self) -> NodeStatusObservation {
        let started_at = Instant::now();
        let headers = self
            .url
            .headers
            .clone()
            .unwrap_or_default()
            .into_iter()
            .try_fold(HeaderMap::new(), |mut headers, (name, value)| {
                let name = HeaderName::from_str(&name).map_err(|error| error.to_string())?;
                let value = HeaderValue::from_str(&value).map_err(|error| error.to_string())?;
                headers.insert(name, value);
                Ok::<_, String>(headers)
            });
        let headers = match headers {
            Ok(headers) => headers,
            Err(error) => return NodeStatusObservation::new(self.url.clone(), NodeStatusState::error(error.to_string()), started_at.elapsed()),
        };
        let client = match gem_client::builder().default_headers(headers).build() {
            Ok(client) => client,
            Err(error) => return NodeStatusObservation::new(self.url.clone(), NodeStatusState::error(error.to_string()), started_at.elapsed()),
        };
        let provider = ProviderFactory::new_provider_with_client(self.config.clone(), "dynode_get_status", client);
        match provider.get_node_status().await {
            Ok(status) => {
                let observation = NodeStatusObservation::new(self.url.clone(), NodeStatusState::healthy(status), started_at.elapsed());
                let Some(profile) = self.profile else {
                    return observation;
                };
                let report = provider.check_node(profile, &NoopNodeCheckReporter).await;
                if report.is_healthy() {
                    observation
                } else {
                    NodeStatusObservation::new(
                        self.url.clone(),
                        NodeStatusState::error(report.error().unwrap_or_else(|| "node check failed".to_string())),
                        started_at.elapsed(),
                    )
                }
            }
            Err(error) => NodeStatusObservation::new(self.url.clone(), NodeStatusState::error(error.to_string()), started_at.elapsed())
                .with_error_kind(CurrentNodeErrorKind::from_error(error.as_ref())),
        }
    }
}
