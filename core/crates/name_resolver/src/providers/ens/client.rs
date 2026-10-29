use std::error::Error;

use alloy_ens::{UNIVERSAL_RESOLVER_ADDRESS, coin_type, namehash, try_dns_encode};
use alloy_primitives::{Address, Bytes, U256, hex};
use alloy_sol_types::{SolCall, SolInterface};
use gem_client::Client;
use gem_evm::jsonrpc::EthereumRpc;
use gem_jsonrpc::JsonRpcClient;
use gem_jsonrpc::types::TypedJsonRpcError;
use primitives::Chain;
use serde::Deserialize;

use super::contract::{ENSMulticoinResolver, ENSResolver, UniversalResolver, UniversalResolver::UniversalResolverErrors};

#[derive(Debug, Default, Deserialize)]
struct RevertData {
    data: Option<String>,
}

type EnsRpcError = TypedJsonRpcError<RevertData>;

pub struct EnsClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
}

impl<C: Client + Clone> EnsClient<C> {
    pub fn new(client: C) -> Self {
        Self { client: JsonRpcClient::new(client) }
    }

    pub async fn get_address(&self, name: &str, chain: Chain) -> Result<Option<Address>, Box<dyn Error + Send + Sync>> {
        let node = namehash(name);
        let coin_type = chain.network_id_value().and_then(coin_type::evm_chain).ok_or("ENS has no coin type for this network")?;
        if coin_type == coin_type::ETH {
            let Some(result) = self.resolve(name, ENSResolver::addrCall { node }.abi_encode()).await? else {
                return Ok(None);
            };
            return Ok(Some(ENSResolver::addrCall::abi_decode_returns(&result)?));
        }
        let call = ENSMulticoinResolver::addrCall { node, coinType: U256::from(coin_type) };
        let Some(result) = self.resolve(name, call.abi_encode()).await? else {
            return Ok(None);
        };
        match ENSMulticoinResolver::addrCall::abi_decode_returns(&result)?.as_ref() {
            [] => Ok(None),
            address if address.len() == Address::len_bytes() => Ok(Some(Address::from_slice(address))),
            _ => Err("ENS address record is not an EVM address".into()),
        }
    }

    async fn resolve(&self, name: &str, data: Vec<u8>) -> Result<Option<Bytes>, Box<dyn Error + Send + Sync>> {
        let call = UniversalResolver::resolveCall {
            name: try_dns_encode(name)?.into(),
            data: data.into(),
        }
        .abi_encode();
        match self.client.request_with_error::<String, _, EnsRpcError>(EthereumRpc::call(&UNIVERSAL_RESOLVER_ADDRESS.to_string(), call)).await {
            Ok(result) => Ok(Some(UniversalResolver::resolveCall::abi_decode_returns(&hex::decode(result)?)?._0)),
            Err(error) => match resolver_error(&error) {
                Some(UniversalResolverErrors::ResolverNotFound(_) | UniversalResolverErrors::UnsupportedResolverProfile(_) | UniversalResolverErrors::OffchainLookup(_)) => Ok(None),
                None => Err(error.into()),
            },
        }
    }
}

fn resolver_error(error: &EnsRpcError) -> Option<UniversalResolverErrors> {
    let data = hex::decode(error.payload.data.as_ref()?).ok()?;
    UniversalResolverErrors::abi_decode(&data).ok()
}
