use std::error::Error;

use alloy_ens::{UNIVERSAL_RESOLVER_ADDRESS, namehash, try_dns_encode};
use alloy_primitives::{Address, hex};
use alloy_sol_types::{SolCall, SolInterface};
use gem_client::Client;
use gem_evm::jsonrpc::EthereumRpc;
use gem_jsonrpc::JsonRpcClient;
use gem_jsonrpc::types::JsonRpcError;

use super::contract::{ENSResolver, UniversalResolver, UniversalResolver::UniversalResolverErrors};

pub struct EnsClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
}

impl<C: Client + Clone> EnsClient<C> {
    pub fn new(client: C) -> Self {
        Self { client: JsonRpcClient::new(client) }
    }

    pub async fn get_address(&self, name: &str) -> Result<Option<Address>, Box<dyn Error + Send + Sync>> {
        let call = UniversalResolver::resolveCall {
            name: try_dns_encode(name)?.into(),
            data: ENSResolver::addrCall { node: namehash(name) }.abi_encode().into(),
        }
        .abi_encode();
        match self.client.request::<String, _>(EthereumRpc::call(&UNIVERSAL_RESOLVER_ADDRESS.to_string(), call)).await {
            Ok(result) => {
                let resolved = UniversalResolver::resolveCall::abi_decode_returns(&hex::decode(result)?)?;
                Ok(Some(ENSResolver::addrCall::abi_decode_returns(&resolved._0)?))
            }
            Err(error) => match resolver_error(&error) {
                Some(UniversalResolverErrors::ResolverNotFound(_) | UniversalResolverErrors::OffchainLookup(_)) => Ok(None),
                None => Err(error.into()),
            },
        }
    }
}

fn resolver_error(error: &JsonRpcError) -> Option<UniversalResolverErrors> {
    let data = hex::decode(error.data.as_ref()?.as_str()?).ok()?;
    UniversalResolverErrors::abi_decode(&data).ok()
}
