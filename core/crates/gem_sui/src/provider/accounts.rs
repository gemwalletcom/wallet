#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainTraits};
use primitives::Chain;

use crate::rpc::SuiProvider;

#[cfg(feature = "rpc")]
impl ChainTraits for SuiProvider {}

#[cfg(feature = "rpc")]
impl ChainProvider for SuiProvider {
    fn get_chain(&self) -> Chain {
        Chain::Sui
    }
}

#[cfg(feature = "rpc")]
impl ChainAccount for SuiProvider {}

#[cfg(feature = "rpc")]
impl ChainPerpetual for SuiProvider {}

#[cfg(feature = "rpc")]
impl ChainAddressStatus for SuiProvider {}
