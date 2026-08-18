mod broadcast_providers;
mod chain_providers;
mod node_check;
mod provider_config;

use std::{collections::HashMap, sync::Arc};

use chain_traits::ChainTraits;
use gem_algorand::rpc::{AlgorandClient, AlgorandIndexer, AlgorandProvider};
use gem_aptos::rpc::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_bsc::{BscStakingClient, SmartChainStakingParser};
use gem_cardano::rpc::CardanoClient;
use gem_client::{Client as GemHttpClient, ReqwestClient, retry_policy};
use gem_cosmos::rpc::client::CosmosClient;
use gem_everstake::{EverstakeParser, EverstakeStakingClient};
use gem_evm::rpc::{EVMAssetBalanceProvider, EVMIndexer, EVMTransactionsByAddressProvider, EthereumClient, EthereumProvider, EvmProviderExtensions, alchemy_url};
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_monad::{MonadStakingClient, MonadStakingParser};
use gem_near::rpc::{NearClient, NearIndexer, NearProvider};
use gem_optimism::OptimismGasOracle;
use gem_polkadot::rpc::{PolkadotClient, PolkadotIndexer, PolkadotProvider};
use gem_solana::rpc::{SolanaClient, SolanaIndexer, SolanaProvider};
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::{SuiClient, SuiIndexer, SuiProvider};
use gem_ton::rpc::TonClient;
use gem_tron::rpc::{TronProvider, client::TronClient, trongrid::client::TronGridClient};
use gem_xrp::rpc::XrpClient;
use reqwest::Client;

use primitives::{BitcoinChain, Chain, ChainStack, ChainType, EVMChain, chain_cosmos::CosmosChain};
use settings::Settings;

pub use broadcast_providers::BroadcastProviders;
pub use chain_providers::ChainProviders;
pub use chain_traits::{TransactionFeeEstimate, TransactionFeeEstimates, TransactionIdRequest, TransactionsRequest, TransactionsResult};
pub use node_check::node_check_request;
pub use provider_config::ProviderConfig;

// Keep in sync with evm_provider_extensions in gemstone/src/gateway/chain_factory.rs
fn evm_provider_extensions<C: GemHttpClient + Clone + 'static>(chain: EVMChain, client: &EthereumClient<C>) -> EvmProviderExtensions {
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

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings_with_user_agent(chain: Chain, settings: &Settings, user_agent: &str) -> Box<dyn ChainTraits> {
        let url = Self::get_chain_url(chain, settings);

        Self::new_provider(ProviderConfig::from_settings(chain, &url, settings), user_agent)
    }

    pub(crate) fn new_providers_with_user_agent(settings: &Settings, user_agent: &str) -> Vec<Box<dyn ChainTraits>> {
        Chain::all()
            .iter()
            .map(|chain| Self::new_from_settings_with_user_agent(*chain, settings, user_agent))
            .collect()
    }

    fn new_provider(config: ProviderConfig, user_agent: &str) -> Box<dyn ChainTraits> {
        let host = config.url.parse::<url::Url>().ok().and_then(|url| url.host_str().map(String::from)).unwrap_or_default();
        let reqwest_client = gem_client::builder().retry(retry_policy(host, 3)).build().expect("Failed to build reqwest client");
        Self::build_provider(config, user_agent, reqwest_client)
    }

    pub fn new_provider_with_client(config: ProviderConfig, reqwest_client: Client) -> Box<dyn ChainTraits> {
        Self::build_provider(config, "", reqwest_client)
    }

    fn build_provider(config: ProviderConfig, user_agent: &str, reqwest_client: Client) -> Box<dyn ChainTraits> {
        let gem_client = ReqwestClient::new_with_user_agent(config.url.clone(), reqwest_client, user_agent.to_string());
        let chain = config.chain;

        match chain.chain_type() {
            ChainType::Bitcoin => Box::new(BitcoinClient::new(gem_client, BitcoinChain::from_chain(chain).unwrap())),
            ChainType::Ethereum => {
                let evm_chain = EVMChain::from_chain(chain).unwrap();
                let rpc_client = JsonRpcClient::new(gem_client.clone());
                let client = EthereumClient::new(rpc_client, evm_chain);
                let indexer = EVMIndexer::for_chain(
                    gem_client.clone().with_request_timeout(config.indexers.alchemy.timeout).with_base_url(alchemy_url(
                        chain,
                        &config.indexers.alchemy.url,
                        &config.indexers.alchemy.key,
                    )),
                    gem_client.clone().with_request_timeout(config.indexers.ankr.timeout).with_base_url(format!(
                        "{}/{}",
                        config.indexers.ankr.url.trim_end_matches('/'),
                        config.indexers.ankr.key
                    )),
                    config.indexers.blockscout.configure_client(gem_client),
                    config.indexers.blockscout.key,
                    evm_chain,
                );
                let extensions = evm_provider_extensions(evm_chain, &client);
                let provider = if let Some(indexer) = indexer {
                    let indexer = Arc::new(indexer);
                    EthereumProvider::new(
                        client,
                        Box::new(EVMTransactionsByAddressProvider::new(indexer.clone())),
                        Box::new(EVMAssetBalanceProvider::new(indexer)),
                        extensions,
                    )
                } else {
                    EthereumProvider::new_rpc_only_with_extensions(client, extensions)
                };
                Box::new(provider)
            }
            ChainType::Cardano => Box::new(CardanoClient::new(gem_client)),
            ChainType::Cosmos => {
                let chain = CosmosChain::from_chain(chain).unwrap();
                Box::new(CosmosClient::new(chain, gem_client))
            }
            ChainType::Aptos => Box::new(AptosClient::new(gem_client)),
            ChainType::Sui => Box::new(SuiProvider::new(
                SuiClient::new(config.url),
                Box::new(SuiIndexer::new(config.indexers.sui.configure_client(gem_client))),
            )),
            ChainType::Xrp => Box::new(XrpClient::new(JsonRpcClient::new(gem_client))),
            ChainType::Algorand => Box::new(AlgorandProvider::new(
                AlgorandClient::new(gem_client.clone()),
                Box::new(AlgorandIndexer::new(config.indexers.algorand.configure_client(gem_client))),
            )),
            ChainType::Stellar => Box::new(StellarClient::new(gem_client)),
            ChainType::Near => {
                let fastnear_client = config.indexers.fastnear.configure_client(gem_client.clone());
                Box::new(NearProvider::new(
                    NearClient::new(JsonRpcClient::new(gem_client)),
                    Box::new(NearIndexer::new(
                        fastnear_client.clone().with_base_url(config.indexers.fastnear.url.replace("{service}", "transfers")),
                        fastnear_client.with_base_url(config.indexers.fastnear.url.replace("{service}", "tx")),
                    )),
                ))
            }
            ChainType::Polkadot => Box::new(PolkadotProvider::new(
                PolkadotClient::new(gem_client.clone()),
                Box::new(PolkadotIndexer::new(
                    config
                        .indexers
                        .subscan
                        .configure_client(gem_client)
                        .with_default_headers(HashMap::from([("x-api-key".to_string(), config.indexers.subscan.key)])),
                )),
            )),
            ChainType::Solana => Box::new(SolanaProvider::new(
                SolanaClient::new(JsonRpcClient::new(gem_client.clone())),
                Box::new(SolanaIndexer::new(JsonRpcClient::new(
                    gem_client
                        .with_request_timeout(config.indexers.alchemy.timeout)
                        .with_base_url(alchemy_url(chain, &config.indexers.alchemy.url, &config.indexers.alchemy.key)),
                ))),
            )),
            ChainType::Ton => Box::new(TonClient::new(gem_client)),
            ChainType::Tron => {
                let trongrid = TronGridClient::new(config.indexers.trongrid.configure_client(gem_client.clone()), config.indexers.trongrid.key);
                Box::new(TronProvider::new(TronClient::new(gem_client), Box::new(trongrid.clone()), Box::new(trongrid)))
            }
            ChainType::HyperCore => Box::new(HyperCoreClient::new(gem_client)),
        }
    }

    pub fn get_chain_config(chain: Chain, settings: &Settings) -> &settings::Chain {
        match chain {
            Chain::Bitcoin => &settings.chains.bitcoin,
            Chain::BitcoinCash => &settings.chains.bitcoincash,
            Chain::Litecoin => &settings.chains.litecoin,
            Chain::Ethereum => &settings.chains.ethereum,
            Chain::SmartChain => &settings.chains.smartchain,
            Chain::Solana => &settings.chains.solana,
            Chain::Polygon => &settings.chains.polygon,
            Chain::Thorchain => &settings.chains.thorchain,
            Chain::Mayachain => &settings.chains.mayachain,
            Chain::Cosmos => &settings.chains.cosmos,
            Chain::Osmosis => &settings.chains.osmosis,
            Chain::Arbitrum => &settings.chains.arbitrum,
            Chain::Ton => &settings.chains.ton,
            Chain::Tron => &settings.chains.tron,
            Chain::Doge => &settings.chains.doge,
            Chain::Zcash => &settings.chains.zcash,
            Chain::Optimism => &settings.chains.optimism,
            Chain::Aptos => &settings.chains.aptos,
            Chain::Base => &settings.chains.base,
            Chain::AvalancheC => &settings.chains.avalanchec,
            Chain::Sui => &settings.chains.sui,
            Chain::Xrp => &settings.chains.xrp,
            Chain::OpBNB => &settings.chains.opbnb,
            Chain::Fantom => &settings.chains.fantom,
            Chain::Gnosis => &settings.chains.gnosis,
            Chain::Celestia => &settings.chains.celestia,
            Chain::Injective => &settings.chains.injective,
            Chain::Sei => &settings.chains.sei,
            Chain::SeiEvm => &settings.chains.seievm,
            Chain::Manta => &settings.chains.manta,
            Chain::Blast => &settings.chains.blast,
            Chain::Noble => &settings.chains.noble,
            Chain::ZkSync => &settings.chains.zksync,
            Chain::Linea => &settings.chains.linea,
            Chain::Mantle => &settings.chains.mantle,
            Chain::Celo => &settings.chains.celo,
            Chain::Near => &settings.chains.near,
            Chain::World => &settings.chains.world,
            Chain::Plasma => &settings.chains.plasma,
            Chain::Stellar => &settings.chains.stellar,
            Chain::Sonic => &settings.chains.sonic,
            Chain::Algorand => &settings.chains.algorand,
            Chain::Polkadot => &settings.chains.polkadot,
            Chain::Cardano => &settings.chains.cardano,
            Chain::Abstract => &settings.chains.abstract_chain,
            Chain::Berachain => &settings.chains.berachain,
            Chain::Ink => &settings.chains.ink,
            Chain::Unichain => &settings.chains.unichain,
            Chain::Hyperliquid => &settings.chains.hyperliquid,
            Chain::HyperCore => &settings.chains.hypercore,
            Chain::Monad => &settings.chains.monad,
            Chain::XLayer => &settings.chains.xlayer,
            Chain::Robinhood => &settings.chains.robinhood,
            Chain::Stable => &settings.chains.stable,
        }
    }

    pub fn get_chain_endpoints(settings: &Settings) -> HashMap<Chain, String> {
        Chain::all()
            .into_iter()
            .map(|chain| (chain, Self::get_chain_url(chain, settings)))
            .filter(|(_, url)| !url.is_empty())
            .collect()
    }

    pub fn get_chain_url(chain: Chain, settings: &Settings) -> String {
        if settings.dynode.url.is_empty() {
            Self::get_chain_config(chain, settings).url.clone()
        } else {
            format!("{}/{}", settings.dynode.url, chain.as_ref())
        }
    }
}
