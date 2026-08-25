#![allow(unused)]

use std::{collections::HashMap, env, path::PathBuf, time::Duration};

use config::{Config, ConfigError, Environment, File};
use gem_client::RemoteProviderConfig;
use serde::Deserialize;
use serde_serializers::duration;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub dynode: URL,
    pub redis: Redis,
    pub postgres: Postgres,
    pub meilisearch: MeiliSearch,
    pub rabbitmq: RabbitMQ,

    pub api: API,
    pub parser: Parser,
    pub daemon: Daemon,
    pub consumer: Consumer,

    pub fiat: Fiat,

    pub swap: Swap,

    pub prices: Prices,
    pub defi: Defi,
    pub coingecko: CoinGecko,
    pub coinmarketcap: CoinMarketCap,
    pub name: Name,
    pub chains: Chains,
    pub pusher: Pusher,
    pub security: Security,
    pub support: Support,
    pub nft: NFT,
    pub indexer: Indexer,
    pub assets: Assets,
    pub rewards: Rewards,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Indexer {
    pub alchemy: ProviderSettings,
    pub algorand: ProviderSettings,
    pub ankr: ProviderSettings,
    pub blockscout: ProviderSettings,
    pub fastnear: FastNearIndexer,
    pub subscan: ProviderSettings,
    pub sui: ProviderSettings,
    pub ton: ProviderSettings,
    pub trongrid: ProviderSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FastNearIndexer {
    pub neardata: ProviderSettings,
    pub transfers: ProviderSettings,
    pub tx: ProviderSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProviderSettings {
    pub url: String,
    pub key: KeySecret,
}

impl ProviderSettings {
    pub fn remote_provider_config(&self) -> RemoteProviderConfig {
        RemoteProviderConfig {
            url: self.url.clone(),
            key: self.key.secret.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fiat {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub moonpay: MoonPay,
    pub transak: Transak,
    pub mercuryo: Mercuryo,
    pub banxa: Banxa,
    pub paybis: Paybis,
    pub flashnet: Flashnet,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Redis {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Postgres {
    pub url: String,
    pub pool: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Retry {
    #[serde(deserialize_with = "duration::deserialize")]
    pub delay: Duration,
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RabbitMQ {
    pub url: String,
    pub prefetch: u16,
    pub retry: Retry,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MeiliSearch {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeySecret {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Key {
    pub secret: String,
    pub public: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MoonPay {
    pub url: String,
    pub key: Key,
    pub webhook: SecretKeySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transak {
    pub url: String,
    pub gateway: URL,
    pub key: Key,
    pub referrer: Referrer,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Referrer {
    pub domain: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mercuryo {
    pub url: String,
    pub key: MercuryoKey,
    pub webhook: SecretKeySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Flashnet {
    pub url: String,
    pub key: Key,
    pub webhook: SecretKeySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Banxa {
    pub api: URL,
    pub redirect: URL,
    pub partner: String,
    pub key: KeySecret,
    pub webhook: SecretKeySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Paybis {
    pub url: String,
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MercuryoKey {
    pub secret: String,
    pub public: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecretKeySettings {
    pub key: KeySecret,
}
pub type CoinGecko = ProviderSettings;
pub type CoinMarketCap = ProviderSettings;

#[derive(Debug, Deserialize, Clone)]
pub struct UrlSecretKeySettings {
    pub url: String,
    pub key: KeySecret,
}
#[derive(Debug, Deserialize, Clone)]
pub struct UrlKeySettings {
    pub url: String,
    pub key: Key,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Prices {
    pub coingecko: CoinGecko,
    pub pyth: ProviderSettings,
    pub jupiter: ProviderSettings,
    pub defillama: ProviderSettings,
    pub tonapi: ProviderSettings,
    pub stonfi: ProviderSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Defi {
    pub jupiter: ProviderSettings,
    pub zerion: ProviderSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Name {
    pub max_name_length: usize,
    pub ens: URL,
    pub ud: URL,
    pub sns: URL,
    pub ton: URL,
    pub eths: URL,
    pub spaceid: URL,
    pub did: URL,
    pub suins: URL,
    pub aptos: URL,
    pub injective: URL,
    pub icns: URL,
    pub lens: URL,
    pub base: URL,
    pub hyperliquid: URL,
    pub alldomains: URL,
    pub near: URL,
}

#[derive(Debug, Deserialize, Clone)]
pub struct URL {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chains {
    pub solana: Chain,
    pub ethereum: Chain,
    pub smartchain: Chain,
    pub polygon: Chain,
    pub optimism: Chain,
    pub arbitrum: Chain,
    pub base: Chain,
    pub opbnb: Chain,
    pub avalanchec: Chain,
    pub ton: Chain,
    pub cosmos: Chain,
    pub osmosis: Chain,
    pub thorchain: Chain,
    pub mayachain: Chain,
    pub celestia: Chain,
    pub tron: Chain,
    pub xrp: Chain,
    pub aptos: Chain,
    pub sui: Chain,
    pub bitcoin: Chain,
    pub bitcoincash: Chain,
    pub litecoin: Chain,
    pub doge: Chain,
    pub zcash: Chain,
    pub fantom: Chain,
    pub gnosis: Chain,
    pub injective: Chain,
    pub sei: Chain,
    pub seievm: Chain,
    pub manta: Chain,
    pub blast: Chain,
    pub noble: Chain,
    pub zksync: Chain,
    pub linea: Chain,
    pub mantle: Chain,
    pub celo: Chain,
    pub near: Chain,
    pub world: Chain,
    pub plasma: Chain,
    pub stellar: Chain,
    pub sonic: Chain,
    pub algorand: Chain,
    pub polkadot: Chain,
    pub cardano: Chain,
    #[serde(rename = "abstract")]
    pub abstract_chain: Chain,
    pub berachain: Chain,
    pub ink: Chain,
    pub unichain: Chain,
    pub hyperliquid: Chain,
    pub hypercore: Chain,
    pub monad: Chain,
    pub xlayer: Chain,
    pub robinhood: Chain,
    pub stable: Chain,
    pub tempo: Chain,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Chain {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Shutdown {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Parser {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub shutdown: Shutdown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Daemon {
    pub service: String,
    pub shutdown: Shutdown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Consumer {
    pub error: ConsumerError,
    #[serde(default, deserialize_with = "duration::deserialize")]
    pub delay: Duration,
    pub shutdown: Shutdown,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConsumerError {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub skip: bool,
    pub retries: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct API {
    pub service: String,
    pub auth: Auth,
    pub admin: Admin,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Auth {
    pub enabled: bool,
    #[serde(deserialize_with = "duration::deserialize")]
    pub tolerance: Duration,
    pub jwt: Jwt,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwt {
    pub secret: String,
    #[serde(deserialize_with = "duration::deserialize")]
    pub expiry: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Admin {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pusher {
    pub url: String,
    pub ios: PusherIOS,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PusherIOS {
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Security {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
    pub abuseipdb: UrlSecretKeySettings,
    pub goplus: URL,
    pub hashdit: UrlKeySettings,
    pub ipapi: UrlSecretKeySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Support {
    pub url: String,
    pub widget: SupportWidget,
    pub webhook: SecretKeySettings,
    pub types: SupportTypes,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SupportWidget {
    pub ios: String,
    pub android: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SupportTypes {
    pub images: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        Self::new_setting_path(current_dir.join("Settings.yaml"))
    }

    pub fn new_setting_path(path: PathBuf) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::from(path))
            .add_source(Environment::with_prefix("").prefix_separator("").separator("_"))
            .build()?;
        s.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NFT {
    pub url: String,
    pub alchemy: ProviderSettings,
    pub magiceden: ProviderSettings,
    pub opensea: ProviderSettings,
    pub ton: ProviderSettings,
    pub offchain: NFTOffchain,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NFTOffchain {
    pub timeout: u64,
    pub concurrency: usize,
    pub limit: usize,
}

pub type Assets = URL;

#[derive(Debug, Deserialize, Clone)]
pub struct Rewards {
    #[serde(default)]
    pub wallets: HashMap<String, RewardsWallet>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RewardsWallet {
    pub key: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Swap {
    pub nearintents: URL,
    pub okx: Okx,
    pub swapsxyz: URL,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Okx {
    pub url: String,
    pub key: Key,
    pub passphrase: String,
    pub project: String,
}

#[cfg(feature = "testkit")]
pub mod testkit;

pub fn service_user_agent(service: &str, sub_service: Option<&str>) -> String {
    match sub_service {
        Some(sub) => format!("{}_{}", service, sub),
        None => service.to_string(),
    }
}
