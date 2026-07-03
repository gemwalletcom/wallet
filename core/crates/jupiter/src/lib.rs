pub mod client;
pub mod model;

pub use self::model::*;

use gem_client::ReqwestClient;
pub type JupiterClient = client::JupiterClient<ReqwestClient>;
