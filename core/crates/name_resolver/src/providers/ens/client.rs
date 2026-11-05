use std::error::Error;

use alloy_primitives::{Address, Bytes, U256, hex};
use alloy_sol_types::{SolCall, SolInterface};
use gem_client::Client;
use gem_evm::jsonrpc::EthereumRpc;
use gem_jsonrpc::JsonRpcClient;
use gem_jsonrpc::types::TypedJsonRpcError;
use primitives::Chain;
use serde::Deserialize;

use super::contract::{ENSMulticoinResolver, ENSResolver, UniversalResolver, UniversalResolver::UniversalResolverErrors};
use super::name::{dns_encode, namehash};

const UNIVERSAL_RESOLVER_ADDRESS: &str = "0xeEeEEEeE14D718C2B47D9923Deab1335E144EeEe";
const ETH_COIN_TYPE: u64 = 60;

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
        let coin_type = chain.network_id_value().and_then(evm_coin_type).ok_or("ENS has no coin type for this network")?;
        if coin_type == ETH_COIN_TYPE {
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
            name: dns_encode(name)?.into(),
            data: data.into(),
        }
        .abi_encode();
        match self.client.request_with_error::<String, _, EnsRpcError>(EthereumRpc::call(UNIVERSAL_RESOLVER_ADDRESS, call)).await {
            Ok(result) => Ok(Some(UniversalResolver::resolveCall::abi_decode_returns(&hex::decode(result)?)?._0)),
            Err(error) => match resolver_error(&error) {
                Some(UniversalResolverErrors::ResolverNotFound(_) | UniversalResolverErrors::UnsupportedResolverProfile(_) | UniversalResolverErrors::OffchainLookup(_)) => Ok(None),
                None => Err(error.into()),
            },
        }
    }
}

fn evm_coin_type(chain_id: u64) -> Option<u64> {
    match chain_id {
        1 => Some(ETH_COIN_TYPE),
        0..0x8000_0000 => Some(0x8000_0000 | chain_id),
        _ => None,
    }
}

fn resolver_error(error: &EnsRpcError) -> Option<UniversalResolverErrors> {
    let data = hex::decode(error.payload.data.as_ref()?).ok()?;
    UniversalResolverErrors::abi_decode(&data).ok()
}
