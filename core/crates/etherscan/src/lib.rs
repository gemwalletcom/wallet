pub mod client;
mod model;
mod target;
#[cfg(any(test, debug_assertions))]
pub mod testkit;

pub use model::{GasOracle, Transaction};
pub use target::EtherscanApi;

use gem_client::ReqwestClient;
pub type EtherscanClient = client::EtherscanClient<ReqwestClient>;
