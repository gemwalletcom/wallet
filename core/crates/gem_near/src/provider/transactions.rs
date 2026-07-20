use async_trait::async_trait;
use chain_traits::ChainTransactions;

use gem_client::Client;

use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearClient<C> {}
