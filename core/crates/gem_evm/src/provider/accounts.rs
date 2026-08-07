#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider};
use gem_client::Client;
use primitives::Chain;

use crate::rpc::{EthereumClient, EthereumProvider};

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainProvider for EthereumProvider<C> {
    fn get_chain(&self) -> Chain {
        EthereumClient::get_chain(self)
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainAccount for EthereumProvider<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainPerpetual for EthereumProvider<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainAddressStatus for EthereumProvider<C> {}
