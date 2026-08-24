mod config;
mod gateway;
mod metrics;
mod server;

use std::error::Error;

use crate::config::EgressConfig;
use crate::gateway::Gateway;
use crate::metrics::Metrics;

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = EgressConfig::load()?;
    let address = config.address;
    let port = config.port;
    let request_limit = config.request.limit;
    let metrics = Metrics::new();
    let gateway = Gateway::new(config, metrics.clone())?;
    gateway.start_health_checks();
    server::launch(address, port, request_limit, gateway, metrics).await
}
