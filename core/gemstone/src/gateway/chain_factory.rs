use std::sync::Arc;

use chain_traits::ChainTraits;
use gem_algorand::rpc::{AlgorandClient, AlgorandProvider};
use gem_aptos::rpc::client::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_bsc::BscStakingClient;
use gem_cardano::rpc::client::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_everstake::EverstakeStakingClient;
use gem_evm::rpc::{EthereumClient, EthereumProvider};
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_monad::MonadStakingClient;
use gem_near::rpc::{NearClient, NearProvider};
use gem_optimism::OptimismGasOracle;
use gem_polkadot::rpc::{PolkadotClient, PolkadotProvider};
use gem_solana::rpc::{SolanaClient, SolanaProvider};
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::{SuiClient, SuiProvider};
use gem_tempo::TempoProvider;
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{TronProvider, client::TronClient};
use gem_xrp::rpc::XrpClient;
use primitives::{BitcoinChain, Chain, ChainType, EVMChain, chain_cosmos::CosmosChain};

use super::{GatewayError, PreferencesWrapper, SecureStoreWrapper};
use crate::alien::{AlienProvider, AlienProviderWrapper, new_alien_client};
use crate::network::JsonRpcClient;
use crate::services::preferences::{GemPreferencesStore, GemSecureStore};

pub struct ChainClientFactory {
    alien: Arc<dyn AlienProvider>,
    preferences: Arc<dyn GemPreferencesStore>,
    secure_preferences: Arc<dyn GemSecureStore>,
}

impl ChainClientFactory {
    pub fn new(alien: Arc<dyn AlienProvider>, preferences: Arc<dyn GemPreferencesStore>, secure_preferences: Arc<dyn GemSecureStore>) -> Self {
        Self {
            alien,
            preferences,
            secure_preferences,
        }
    }

    pub async fn create(&self, chain: Chain) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let url = self.alien.get_endpoint(chain).map_err(|e| GatewayError::PlatformError { msg: e.to_string() })?;
        self.create_with_url(chain, url).await
    }

    pub async fn create_with_url(&self, chain: Chain, url: String) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let alien_client = new_alien_client(url.clone(), self.alien.clone());
        match chain.chain_type() {
            ChainType::HyperCore => {
                let preferences = Arc::new(PreferencesWrapper {
                    preferences: self.preferences.clone(),
                });
                let secure_preferences = Arc::new(SecureStoreWrapper {
                    store: self.secure_preferences.clone(),
                });
                Ok(Arc::new(HyperCoreClient::new_with_preferences(alien_client, preferences, secure_preferences)))
            }
            ChainType::Bitcoin => Ok(Arc::new(BitcoinClient::new(alien_client, BitcoinChain::from_chain(chain).unwrap()))),
            ChainType::Cardano => Ok(Arc::new(CardanoClient::new(alien_client))),
            ChainType::Stellar => Ok(Arc::new(StellarClient::new(alien_client))),
            ChainType::Sui => Ok(Arc::new(SuiProvider::new_rpc_only(SuiClient::new_with_transport(
                url,
                Arc::new(AlienGrpcTransport::new(Arc::new(AlienProviderWrapper::new(self.alien.clone())))),
            )))),
            ChainType::Xrp => Ok(Arc::new(XrpClient::new(JsonRpcClient::new(alien_client.clone())))),
            ChainType::Algorand => Ok(Arc::new(AlgorandProvider::new_rpc_only(AlgorandClient::new(alien_client)))),
            ChainType::Near => Ok(Arc::new(NearProvider::new_rpc_only(NearClient::new(JsonRpcClient::new(alien_client))))),
            ChainType::Aptos => Ok(Arc::new(AptosClient::new(alien_client))),
            ChainType::Cosmos => Ok(Arc::new(CosmosClient::new(CosmosChain::from_chain(chain).unwrap(), alien_client))),
            ChainType::Ton => Ok(Arc::new(TonClient::new(alien_client))),
            ChainType::Tron => Ok(Arc::new(TronProvider::new_rpc_only(TronClient::new(alien_client)))),
            ChainType::Polkadot => Ok(Arc::new(PolkadotProvider::new_rpc_only(PolkadotClient::new(alien_client)))),
            ChainType::Solana => {
                let client = JsonRpcClient::new(alien_client.clone());
                Ok(Arc::new(SolanaProvider::new_rpc_only(SolanaClient::new(client))))
            }
            ChainType::Ethereum => {
                let evm_chain = EVMChain::from_chain(chain).unwrap();
                let client = EthereumClient::new(JsonRpcClient::new(alien_client), evm_chain);
                let provider = TempoProvider::new_or_else(client, |client| {
                    if evm_chain.is_opstack() {
                        Box::new(EthereumProvider::new_rpc_only_with_provider(client.clone(), Box::new(OptimismGasOracle::new(client))))
                    } else {
                        Box::new(match evm_chain {
                            EVMChain::Ethereum => EthereumProvider::new_rpc_only_with_provider(client.clone(), Box::new(EverstakeStakingClient::new(client, None))),
                            EVMChain::Monad => EthereumProvider::new_rpc_only_with_provider(client.clone(), Box::new(MonadStakingClient::new(client))),
                            EVMChain::SmartChain => EthereumProvider::new_rpc_only_with_provider(client.clone(), Box::new(BscStakingClient::new(client))),
                            _ => EthereumProvider::new_rpc_only(client),
                        })
                    }
                });
                Ok(Arc::from(provider))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::preferences::EmptyPreferences;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;

    #[test]
    fn test_get_is_token_address_matches_chain_config() {
        let factory = ChainClientFactory::new(Arc::new(TestAlienProvider::with_status(404)), Arc::new(EmptyPreferences {}), Arc::new(EmptyPreferences {}));
        for chain in Chain::all() {
            let client = block_on(factory.create(chain)).unwrap();
            let is_token_supported = chain.default_asset_type().is_some();
            match chain.chain_type().mock_token_id() {
                Some(token_id) => assert_eq!(
                    client.get_is_token_address(token_id),
                    is_token_supported,
                    "{chain}: get_is_token_address({token_id}) disagrees with default_asset_type in chain config"
                ),
                None => assert!(!is_token_supported, "{chain} declares token support in chain config but has no token id fixture"),
            }
        }
    }
}
