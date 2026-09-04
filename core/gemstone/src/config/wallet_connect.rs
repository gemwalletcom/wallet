use primitives::{Chain, EVMChain};

use super::public::PublicUrl;

const PROJECT_ID: &str = "3bc07cd7179d11ea65335fb9377702b6";
const APP_NAME: &str = "Gem Wallet";
const APP_DESCRIPTION: &str = "Gem Web3 Wallet";
const APP_ICON_URL: &str = "https://gemwallet.com/images/gem-logo-256x256.png";

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct WalletConnectConfig {
    pub chains: Vec<String>,
    pub project_id: String,
    pub app_name: String,
    pub app_description: String,
    pub app_url: String,
    pub app_icons: Vec<String>,
}

pub fn get_wallet_connect_config() -> WalletConnectConfig {
    let chains: Vec<Chain> = [
        vec![Chain::Solana, Chain::Sui, Chain::Ton, Chain::Tron],
        EVMChain::all().iter().map(|x| x.to_chain()).collect(),
    ]
    .concat();

    WalletConnectConfig {
        chains: chains.into_iter().map(|x| x.to_string()).collect(),
        project_id: PROJECT_ID.to_string(),
        app_name: APP_NAME.to_string(),
        app_description: APP_DESCRIPTION.to_string(),
        app_url: PublicUrl::Website.url(),
        app_icons: vec![APP_ICON_URL.to_string()],
    }
}
