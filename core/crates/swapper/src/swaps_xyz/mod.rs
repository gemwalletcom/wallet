mod chain;
mod client;
mod model;
mod provider;

pub use model::{ActionRequest, ActionResponse};
pub use provider::SwapsXyz;

use crate::{SwapperProvider, config::get_swap_proxy_url};

pub fn base_url() -> String {
    get_swap_proxy_url(SwapperProvider::SwapsXyz.as_ref())
}
