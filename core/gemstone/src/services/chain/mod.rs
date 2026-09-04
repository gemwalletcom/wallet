pub mod rules;

use primitives::Chain;

use crate::wallet_connect::{wallet_connect_chain, wallet_connect_namespace, wallet_connect_reference};

#[derive(Default, uniffi::Object)]
pub struct GemChainService {}

#[uniffi::export]
impl GemChainService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_chains(&self, query: String) -> Vec<Chain> {
        rules::matching_chains(rules::chains_by_rank(), &query)
    }

    pub fn get_matching_chains(&self, chains: Vec<Chain>, query: String) -> Vec<Chain> {
        rules::matching_chains(chains, &query)
    }

    pub fn caip2_namespace(&self, chain: Chain) -> Option<String> {
        wallet_connect_namespace(chain)
    }

    pub fn caip2_reference(&self, chain: Chain) -> Option<String> {
        wallet_connect_reference(chain)
    }

    pub fn chain_from_caip2(&self, chain_id: String) -> Option<Chain> {
        wallet_connect_chain(chain_id)
    }
}
