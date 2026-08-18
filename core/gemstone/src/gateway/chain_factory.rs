use std::sync::Arc;

use chain_traits::ChainTraits;
use gem_algorand::rpc::{AlgorandClient, AlgorandProvider};
use gem_aptos::rpc::client::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_bsc::{BscStakingClient, SmartChainStakingParser};
use gem_cardano::rpc::client::CardanoClient;
use gem_client::Client;
use gem_cosmos::rpc::client::CosmosClient;
use gem_everstake::{EverstakeParser, EverstakeStakingClient};
use gem_evm::rpc::{EthereumClient, EthereumProvider, EvmProviderExtensions};
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_monad::{MonadStakingClient, MonadStakingParser};
use gem_near::rpc::{NearClient, NearProvider};
use gem_optimism::OptimismGasOracle;
use gem_polkadot::rpc::{PolkadotClient, PolkadotProvider};
use gem_solana::rpc::{SolanaClient, SolanaProvider};
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::{SuiClient, SuiProvider};
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{TronProvider, client::TronClient};
use gem_xrp::rpc::XrpClient;
use primitives::{BitcoinChain, Chain, ChainStack, ChainType, EVMChain, chain_cosmos::CosmosChain};

use super::{GatewayError, GemPreferences, PreferencesWrapper};
use crate::alien::{AlienProvider, AlienProviderWrapper, new_alien_client};
use crate::network::JsonRpcClient;

pub struct ChainClientFactory {
    alien: Arc<dyn AlienProvider>,
    preferences: Arc<dyn GemPreferences>,
    secure_preferences: Arc<dyn GemPreferences>,
}

impl ChainClientFactory {
    pub fn new(alien: Arc<dyn AlienProvider>, preferences: Arc<dyn GemPreferences>, secure_preferences: Arc<dyn GemPreferences>) -> Self {
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
                let secure_preferences = Arc::new(PreferencesWrapper {
                    preferences: self.secure_preferences.clone(),
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
                let extensions = evm_provider_extensions(evm_chain, &client);
                Ok(Arc::new(EthereumProvider::new_rpc_only_with_extensions(client, extensions)))
            }
        }
    }
}

// Keep in sync with evm_provider_extensions in crates/settings_chain/src/lib.rs
fn evm_provider_extensions<C: Client + Clone + 'static>(chain: EVMChain, client: &EthereumClient<C>) -> EvmProviderExtensions {
    let extensions = match chain {
        EVMChain::Ethereum => EvmProviderExtensions {
            staking: Some(Box::new(EverstakeStakingClient::new(client.clone()))),
            parsers: vec![Box::new(EverstakeParser)],
            ..Default::default()
        },
        EVMChain::SmartChain => EvmProviderExtensions {
            staking: Some(Box::new(BscStakingClient::new(client.clone()))),
            parsers: vec![Box::new(SmartChainStakingParser)],
            ..Default::default()
        },
        EVMChain::Monad => EvmProviderExtensions {
            staking: Some(Box::new(MonadStakingClient::new(client.clone()))),
            parsers: vec![Box::new(MonadStakingParser)],
            ..Default::default()
        },
        _ => EvmProviderExtensions::default(),
    };
    match chain.chain_stack() {
        ChainStack::Optimism => EvmProviderExtensions {
            fee_calculator: Some(Box::new(OptimismGasOracle::new(client.clone()))),
            ..extensions
        },
        ChainStack::Native | ChainStack::ZkSync => extensions,
    }
}
