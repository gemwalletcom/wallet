mod chain;
mod client;
mod model;
mod provider;
mod target;
#[cfg(test)]
mod testkit;

pub use model::ActionRequest;
pub(crate) use model::DepositResponse;
pub use provider::SwapsXyz;

use crate::{SwapperProvider, config::get_swap_proxy_url};

fn base_url() -> String {
    get_swap_proxy_url(SwapperProvider::SwapsXyz.as_ref())
}
