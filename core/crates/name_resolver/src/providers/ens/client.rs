use std::error::Error;

use alloy_ens::{UNIVERSAL_RESOLVER_ADDRESS, coin_type, namehash, try_dns_encode};
use alloy_primitives::{Address, Bytes, U256, hex};
use alloy_sol_types::{SolCall, SolError, SolInterface};
use gem_client::Client;
use gem_evm::jsonrpc::EthereumRpc;
use gem_jsonrpc::JsonRpcClient;
use gem_jsonrpc::types::TypedJsonRpcError;
use primitives::Chain;
use serde::Deserialize;

use super::ccip::CcipClient;
use super::contract::{ENSMulticoinResolver, ENSResolver, HttpError, UniversalResolver, UniversalResolver::UniversalResolverErrors};

const MAX_OFFCHAIN_LOOKUPS: usize = 2;

#[derive(Debug, Default, Deserialize)]
struct RevertData {
    data: Option<String>,
}

type EnsRpcError = TypedJsonRpcError<RevertData>;

pub struct EnsClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
    ccip: CcipClient<C>,
}

impl<C: Client + Clone> EnsClient<C> {
    pub fn new(client: C, ccip: CcipClient<C>) -> Self {
        Self { client: JsonRpcClient::new(client), ccip }
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
        let mut call = UniversalResolver::resolveCall {
            name: try_dns_encode(name)?.into(),
            data: data.into(),
        }
        .abi_encode();
        for lookups in 0.. {
            let error = match self.client.request_with_error::<String, _, EnsRpcError>(EthereumRpc::call(&UNIVERSAL_RESOLVER_ADDRESS.to_string(), call)).await {
                Ok(result) => return Ok(Some(UniversalResolver::resolveCall::abi_decode_returns(&hex::decode(result)?)?._0)),
                Err(error) => error,
            };
            let lookup = match resolver_error(&error) {
                Some(UniversalResolverErrors::ResolverNotFound(_) | UniversalResolverErrors::UnsupportedResolverProfile(_)) => return Ok(None),
                Some(UniversalResolverErrors::OffchainLookup(lookup)) => lookup,
                Some(UniversalResolverErrors::ResolverError(resolver_error)) => match HttpError::abi_decode(&resolver_error.errorData) {
                    Ok(HttpError { status, message }) => return Err(format!("ENS offchain lookup failed: HTTP {status} {message}").into()),
                    Err(_) => return Err(error.into()),
                },
                None => return Err(error.into()),
            };
            if lookup.sender != UNIVERSAL_RESOLVER_ADDRESS {
                return Err("ENS offchain lookup sender does not match Universal Resolver".into());
            }
            if lookups == MAX_OFFCHAIN_LOOKUPS {
                break;
            }
            call = lookup.callback(self.ccip.lookup(&lookup).await?);
        }
        Err("ENS offchain lookup limit exceeded".into())
    }
}

fn resolver_error(error: &EnsRpcError) -> Option<UniversalResolverErrors> {
    let data = hex::decode(error.payload.data.as_ref()?).ok()?;
    UniversalResolverErrors::abi_decode(&data).ok()
}
