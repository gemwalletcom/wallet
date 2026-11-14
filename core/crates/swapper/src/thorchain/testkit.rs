use super::{asset::THORChainAsset, chain::ChainName, client::ThorChainSwapClient, provider::ThorChain};
use crate::alien::mock::ProviderMock;
use gem_client::testkit::MockClient;
use primitives::Chain;
use std::sync::Arc;

impl THORChainAsset {
    pub fn mock(chain: Chain) -> Self {
        Self {
            chain: ChainName::from_chain(chain).unwrap(),
            symbol: "TEST".to_string(),
            token_id: None,
            decimals: 18,
        }
    }
}

impl ThorChain<MockClient> {
    pub fn mock(client: MockClient) -> Self {
        Self::with_client(ThorChainSwapClient::new(client), Arc::new(ProviderMock::new(String::new())))
    }
}
