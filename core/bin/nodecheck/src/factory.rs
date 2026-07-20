use std::{error::Error, str::FromStr};

use gem_client::{ReqwestClient, builder};
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_solana::SolanaClient;
use primitives::{Chain, ChainType, EVMChain};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::{
    chains::{ethereum::EthereumNodeChecker, solana::SolanaNodeChecker},
    checker::NodeCheck,
};

pub(crate) fn new_checker(chain: Chain, url: String, headers: &[String]) -> Result<Box<dyn NodeCheck>, Box<dyn Error + Send + Sync>> {
    let headers = headers
        .iter()
        .map(|header| {
            let (name, value) = header.split_once(':').ok_or_else(|| format!("invalid header: {header}"))?;
            Ok((HeaderName::from_str(name.trim())?, HeaderValue::from_str(value.trim())?))
        })
        .collect::<Result<HeaderMap, Box<dyn Error + Send + Sync>>>()?;
    let client = ReqwestClient::new(url, builder().default_headers(headers).build()?);
    match chain.chain_type() {
        ChainType::Ethereum => {
            let chain = EVMChain::from_chain(chain).ok_or_else(|| format!("invalid Ethereum chain: {chain}"))?;
            let client = EthereumClient::new(JsonRpcClient::new(client), chain);
            Ok(Box::new(EthereumNodeChecker::new(client)))
        }
        ChainType::Solana => {
            let client = SolanaClient::new(JsonRpcClient::new(client));
            Ok(Box::new(SolanaNodeChecker::new(client)))
        }
        ChainType::Bitcoin
        | ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => Err(format!("node checking is not supported for {chain}").into()),
    }
}
