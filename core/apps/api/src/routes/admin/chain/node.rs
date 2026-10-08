use std::sync::Arc;

use axum::extract::State;
use services::chain::{NodeStatusResult, NodesStatusClient};

use crate::auth::api_client::ChainRead;
use crate::request::{ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_nodes_status(_permission: ChainRead, Path(chain): Path<ChainParam>, State(client): State<Arc<NodesStatusClient>>) -> ApiResponse<Vec<NodeStatusResult>> {
    client.get_nodes_status(chain.0).await.into()
}
