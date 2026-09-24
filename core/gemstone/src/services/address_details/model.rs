use primitives::{AddressName, AddressType, Chain, VerificationStatus, block_explorer::BlockExplorerLink};

use super::rules;
use crate::models::copy::GemCopy;
use crate::models::list::GemListSection;
use crate::models::state::{GemLoad, GemLoadState};
use crate::services::balance::GemBalanceRow;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddressDetails {
    pub chain: Chain,
    pub address: String,
    pub name: Option<String>,
    pub address_type: Option<AddressType>,
    pub status: VerificationStatus,
    pub copy: GemCopy,
    pub link: BlockExplorerLink,
    pub state: GemLoadState,
    pub balances: Vec<GemBalanceRow>,
}

impl GemAddressDetails {
    pub(super) fn load(&self) -> GemLoad<Vec<GemBalanceRow>> {
        GemLoad {
            state: self.state.clone(),
            value: self.balances.clone(),
        }
    }
}

#[uniffi::export]
impl GemAddressDetails {
    pub fn sections(&self, address_name: Option<AddressName>) -> Vec<GemListSection> {
        rules::sections(self, address_name.as_ref())
    }
}
