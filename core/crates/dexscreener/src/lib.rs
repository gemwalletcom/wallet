pub mod client;
mod mapper;
mod model;
mod target;

pub use mapper::{chain_from_id, chain_id};
pub use model::{Meta, MetaDetails, Pair};

use gem_client::ReqwestClient;
pub type DexScreenerClient = client::DexScreenerClient<ReqwestClient>;
