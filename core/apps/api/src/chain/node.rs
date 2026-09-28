use rocket::{State, get};
use services::chain::{NodeStatusResult, NodesStatusClient};

use crate::api_clients::PermissionChainRead;
use crate::params::ChainParam;
use crate::responders::ApiResponse;

#[get("/chain/nodes/<chain>/status")]
pub async fn get_nodes_status(_permission: PermissionChainRead, chain: ChainParam, client: &State<NodesStatusClient>) -> ApiResponse<Vec<NodeStatusResult>> {
    client.get_nodes_status(chain.0).await.into()
}
