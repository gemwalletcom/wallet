mod chain;
mod client;
mod model;
mod partner_provider;
mod provider;
mod target;
#[cfg(test)]
mod testkit;

pub use model::{ActionRequest, ActionResponse};
pub use partner_provider::SwapsXyzPartnerProvider;
pub use provider::SwapsXyz;

use crate::{SwapperProvider, config::get_swap_proxy_url};

pub fn base_url() -> String {
    get_swap_proxy_url(SwapperProvider::SwapsXyz.as_ref())
}
