use std::ops::Deref;

use chain_traits::{ChainAccount, ChainPerpetual, ChainProvider};
use gem_client::Client;
use primitives::Chain;

use super::SolanaClient;

pub struct SolanaProvider<C: Client + Clone> {
    client: SolanaClient<C>,
}

impl<C: Client + Clone> SolanaProvider<C> {
    pub fn new(client: SolanaClient<C>) -> Self {
        Self { client }
    }
}

impl<C: Client + Clone> Deref for SolanaProvider<C> {
    type Target = SolanaClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C: Client + Clone> ChainProvider for SolanaProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }
}

impl<C: Client + Clone> ChainAccount for SolanaProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for SolanaProvider<C> {}
