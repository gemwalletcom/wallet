pub mod client;
pub mod model;
mod target;

#[cfg(test)]
pub(crate) mod testkit;

pub use self::model::*;

use gem_reqwest::ReqwestClient;
pub type JupiterClient = client::JupiterClient<ReqwestClient>;
