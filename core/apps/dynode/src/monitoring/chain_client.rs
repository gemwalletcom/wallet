use std::time::Instant;

use primitives::{Chain, NodeStatusState, NodeType};
use settings_chain::{ProviderConfig, ProviderFactory};

use super::switch_reason::CurrentNodeErrorKind;
use super::sync::NodeStatusObservation;
use crate::config::Url;

pub struct ChainClient {
    config: ProviderConfig,
    url: Url,
}

impl ChainClient {
    pub fn new(chain: Chain, url: Url) -> Self {
        let config = ProviderConfig::new(chain, &url.url, NodeType::Default, "", "");
        Self { config, url }
    }

    pub async fn get_status(&self) -> NodeStatusObservation {
        let started_at = Instant::now();
        match ProviderFactory::new_provider(self.config.clone(), "dynode_get_status").get_node_status().await {
            Ok(status) => NodeStatusObservation::new(self.url.clone(), NodeStatusState::healthy(status), started_at.elapsed()),
            Err(error) => NodeStatusObservation::new(self.url.clone(), NodeStatusState::error(error.to_string()), started_at.elapsed())
                .with_error_kind(CurrentNodeErrorKind::from_error(error.as_ref())),
        }
    }
}
