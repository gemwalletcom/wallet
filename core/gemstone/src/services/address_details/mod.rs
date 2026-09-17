pub mod model;
mod rules;

use std::sync::Arc;

use primitives::Chain;

use crate::gateway::GemGateway;
use crate::models::state::GemLoad;
use crate::services::balance::GemBalanceRow;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
pub use model::GemAddressDetails;

#[derive(uniffi::Object)]
pub struct GemAddressDetailsService {
    gateway: Arc<GemGateway>,
    explorer: Arc<GemExplorerService>,
    names: Arc<GemNameService>,
}

#[uniffi::export]
impl GemAddressDetailsService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, explorer: Arc<GemExplorerService>, names: Arc<GemNameService>) -> Self {
        Self { gateway, explorer, names }
    }

    pub fn details(&self, chain: Chain, address: String) -> GemAddressDetails {
        let link = self.explorer.get_address_url(chain, address.clone());
        rules::details(chain, address, None, link, GemLoad::loading())
    }

    pub async fn refresh(&self, details: GemAddressDetails) -> GemAddressDetails {
        let chain = details.chain;
        let (balances, name) = futures::join!(self.fetch_balances(chain, details.address.clone()), self.names.address_name(chain, details.address.clone()));
        let link = self.explorer.get_address_url(chain, details.address.clone());
        rules::details(
            chain,
            details.address.clone(),
            rules::display_name(name.ok().flatten(), &details.address),
            link,
            details.load().data(balances),
        )
    }
}

impl GemAddressDetailsService {
    async fn fetch_balances(&self, chain: Chain, address: String) -> Result<Vec<GemBalanceRow>, GemServiceError> {
        let (coin, stake) = futures::try_join!(self.gateway.get_balance_coin(chain, address.clone()), self.gateway.get_balance_staking(chain, address))?;
        Ok(rules::balance_rows(chain, coin, stake))
    }
}
