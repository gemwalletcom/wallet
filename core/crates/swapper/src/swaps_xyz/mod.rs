mod chain;
mod client;
mod model;
mod provider;

pub use model::{ActionRequest, ActionResponse};
pub use provider::SwapsXyz;

use crate::config::get_swap_proxy_url;

const NATIVE_TOKEN: &str = "0x0000000000000000000000000000000000000000";

pub fn base_url() -> String {
    get_swap_proxy_url("swaps_xyz")
}
