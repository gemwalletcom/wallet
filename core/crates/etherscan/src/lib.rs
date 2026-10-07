pub mod client;
mod model;
mod target;

pub use model::GasOracle;

use gem_client::ReqwestClient;
pub type EtherscanClient = client::EtherscanClient<ReqwestClient>;
