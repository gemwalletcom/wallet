use std::str::FromStr;

use dynode::BoxError;
use dynode::config::load_config;
use dynode::server::Server;
use primitives::Chain;

#[rocket::main]
async fn main() -> Result<(), BoxError> {
    let selected_chain = std::env::args().nth(1).map(|value| Chain::from_str(&value)).transpose()?;
    let (config, mut chains) = load_config()?;
    if let Some(chain) = selected_chain {
        if !chains.contains_key(&chain) {
            return Err(format!("Chain is not configured: {chain}").into());
        }
        chains.retain(|configured, _| *configured == chain);
    }
    Server::new(config, chains)?.launch().await
}
