use async_trait::async_trait;
use chain_traits::ChainToken;

use gem_client::Client;

use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainToken for CardanoClient<C> {}
