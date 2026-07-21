use super::{GatewayError, GemPreferences, PreferencesWrapper};
use crate::alien::{AlienProvider, AlienProviderWrapper, new_alien_client};
use crate::network::JsonRpcClient;
use chain_traits::ChainTraits;
use gem_algorand::rpc::client::AlgorandClient;
use gem_algorand::rpc::{ALGORAND_INDEXER_URL, AlgorandIndexer};
use gem_aptos::rpc::client::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_cardano::rpc::client::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_evm::rpc::EthereumClient;
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_jsonrpc::grpc::AlienGrpcTransport;
use gem_near::rpc::{FASTNEAR_TRANSACTIONS_URL, FASTNEAR_TRANSFERS_URL, NearClient, NearIndexer};
use gem_polkadot::rpc::{POLKADOT_ASSET_HUB_SUBSCAN_URL, PolkadotClient, PolkadotIndexer};
use gem_solana::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::{SUI_GRAPHQL_URL, SuiClient, SuiIndexer};
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use gem_xrp::rpc::XrpClient;
use primitives::{BitcoinChain, Chain, EVMChain, chain_cosmos::CosmosChain};
use std::sync::Arc;

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
        match chain {
            Chain::HyperCore => {
                let preferences = Arc::new(PreferencesWrapper {
                    preferences: self.preferences.clone(),
                });
                let secure_preferences = Arc::new(PreferencesWrapper {
                    preferences: self.secure_preferences.clone(),
                });
                Ok(Arc::new(HyperCoreClient::new_with_preferences(alien_client, preferences, secure_preferences)))
            }
            Chain::Bitcoin | Chain::BitcoinCash | Chain::Litecoin | Chain::Doge | Chain::Zcash => {
                Ok(Arc::new(BitcoinClient::new(alien_client, BitcoinChain::from_chain(chain).unwrap())))
            }
            Chain::Cardano => Ok(Arc::new(CardanoClient::new(alien_client))),
            Chain::Stellar => Ok(Arc::new(StellarClient::new(alien_client))),
            Chain::Sui => {
                let indexer_client = new_alien_client(SUI_GRAPHQL_URL.to_string(), self.alien.clone());
                Ok(Arc::new(SuiClient::new_with_transport(
                    url,
                    Arc::new(AlienGrpcTransport::new(Arc::new(AlienProviderWrapper::new(self.alien.clone())))),
                    SuiIndexer::new(indexer_client),
                )))
            }
            Chain::Xrp => Ok(Arc::new(XrpClient::new(JsonRpcClient::new(alien_client.clone())))),
            Chain::Algorand => {
                let indexer_client = new_alien_client(ALGORAND_INDEXER_URL.to_string(), self.alien.clone());
                Ok(Arc::new(AlgorandClient::new(alien_client, AlgorandIndexer::new(indexer_client))))
            }
            Chain::Near => {
                let transfers_client = new_alien_client(FASTNEAR_TRANSFERS_URL.to_string(), self.alien.clone());
                let transactions_client = new_alien_client(FASTNEAR_TRANSACTIONS_URL.to_string(), self.alien.clone());
                Ok(Arc::new(NearClient::new(
                    JsonRpcClient::new(alien_client.clone()),
                    NearIndexer::new(transfers_client, transactions_client),
                )))
            }
            Chain::Aptos => Ok(Arc::new(AptosClient::new(alien_client))),
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Mayachain | Chain::Injective | Chain::Sei | Chain::Noble => {
                Ok(Arc::new(CosmosClient::new(CosmosChain::from_chain(chain).unwrap(), alien_client)))
            }
            Chain::Ton => Ok(Arc::new(TonClient::new(alien_client))),
            Chain::Tron => Ok(Arc::new(TronClient::new(alien_client.clone(), TronGridClient::new(alien_client.clone(), String::new())))),
            Chain::Polkadot => {
                let asset_hub_client = new_alien_client(POLKADOT_ASSET_HUB_SUBSCAN_URL.to_string(), self.alien.clone());
                Ok(Arc::new(PolkadotClient::new(alien_client, PolkadotIndexer::new(asset_hub_client))))
            }
            Chain::Solana => {
                let client = JsonRpcClient::new(alien_client.clone());
                Ok(Arc::new(SolanaClient::new(client)))
            }
            Chain::Ethereum
            | Chain::Arbitrum
            | Chain::SmartChain
            | Chain::Polygon
            | Chain::Optimism
            | Chain::Base
            | Chain::AvalancheC
            | Chain::OpBNB
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::Manta
            | Chain::Blast
            | Chain::ZkSync
            | Chain::Linea
            | Chain::Mantle
            | Chain::Celo
            | Chain::World
            | Chain::Sonic
            | Chain::SeiEvm
            | Chain::Abstract
            | Chain::Berachain
            | Chain::Ink
            | Chain::Unichain
            | Chain::Hyperliquid
            | Chain::Plasma
            | Chain::Monad
            | Chain::XLayer
            | Chain::Robinhood
            | Chain::Stable => Ok(Arc::new(EthereumClient::new(JsonRpcClient::new(alien_client), EVMChain::from_chain(chain).unwrap()))),
        }
    }
}
