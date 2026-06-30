#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainSimulation, ChainTraits};
use primitives::Chain;

use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
impl ChainTraits for SuiClient {}

#[cfg(feature = "rpc")]
impl ChainProvider for SuiClient {
    fn get_chain(&self) -> Chain {
        Chain::Sui
    }
}

#[cfg(feature = "rpc")]
impl ChainAccount for SuiClient {}

#[cfg(feature = "rpc")]
impl ChainPerpetual for SuiClient {}

#[cfg(feature = "rpc")]
impl ChainAddressStatus for SuiClient {}

#[cfg(feature = "rpc")]
impl ChainSimulation for SuiClient {}
