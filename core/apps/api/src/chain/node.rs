use gem_client::DEFAULT_REQUEST_TIMEOUT;
use gem_tracing::{error_fields, error_with_fields};
use primitives::{Chain, NodeStatus, node_config::NodeRegion};
use reqwest::Client;
use rocket::{State, get};
use serde::Serialize;
use settings_chain::{ProviderConfig, ProviderFactory};
use tokio::time::timeout;

use crate::api_clients::PermissionChainRead;
use crate::params::ChainParam;
use crate::responders::ApiResponse;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatusResult {
    region: NodeRegion,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<NodeStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub struct NodesStatusClient {
    client: Client,
}

impl Default for NodesStatusClient {
    fn default() -> Self {
        Self {
            client: gem_client::reqwest_client(),
        }
    }
}

impl NodesStatusClient {
    async fn get_nodes_status(&self, chain: Chain) -> Vec<NodeStatusResult> {
        futures::future::join_all(NodeRegion::all().into_iter().map(|region| {
            let client = self.client.clone();
            async move {
                let provider = ProviderFactory::new_provider_with_client(ProviderConfig::new(chain, &region.url(chain)), client);
                let result = timeout(DEFAULT_REQUEST_TIMEOUT, provider.get_nodes_status()).await;
                match result {
                    Ok(Ok(status)) => NodeStatusResult {
                        region,
                        status: Some(status),
                        error: None,
                    },
                    Ok(Err(error)) => {
                        error_with_fields!("node status check failed", error.as_ref(), chain = chain.as_ref(), region = region.as_ref());
                        NodeStatusResult {
                            region,
                            status: None,
                            error: Some(format!("{} {} node is unavailable", chain.as_ref(), region.as_ref())),
                        }
                    }
                    Err(_) => {
                        error_fields!("node status check timed out", chain = chain.as_ref(), region = region.as_ref());
                        NodeStatusResult {
                            region,
                            status: None,
                            error: Some(format!(
                                "{} {} node timed out after {} seconds",
                                chain.as_ref(),
                                region.as_ref(),
                                DEFAULT_REQUEST_TIMEOUT.as_secs()
                            )),
                        }
                    }
                }
            }
        }))
        .await
    }
}

#[get("/chain/nodes/<chain>/status")]
pub async fn get_nodes_status(_permission: PermissionChainRead, chain: ChainParam, client: &State<NodesStatusClient>) -> ApiResponse<Vec<NodeStatusResult>> {
    client.get_nodes_status(chain.0).await.into()
}

#[cfg(test)]
mod tests {
    use super::NodeStatusResult;
    use primitives::{NodeStatus, node_config::NodeRegion};

    #[test]
    fn test_node_status_result_serialization() {
        let output = serde_json::to_value([
            NodeStatusResult {
                region: NodeRegion::Us,
                status: Some(NodeStatus {
                    chain_id: "bitcoin".to_string(),
                    latest_block_number: 100,
                    latency_ms: 20,
                }),
                error: None,
            },
            NodeStatusResult {
                region: NodeRegion::Eu,
                status: None,
                error: Some("bitcoin eu node is unavailable".to_string()),
            },
        ])
        .unwrap();

        assert_eq!(output[0]["region"], "us");
        assert_eq!(output[0]["status"]["chainId"], "bitcoin");
        assert_eq!(output[0].get("error"), None);
        assert_eq!(output[1]["region"], "eu");
        assert_eq!(output[1]["error"], "bitcoin eu node is unavailable");
        assert_eq!(output[1].get("status"), None);
    }
}
