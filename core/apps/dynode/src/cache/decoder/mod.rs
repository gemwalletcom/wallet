mod evm;
mod sui;

use primitives::ChainType;

use crate::jsonrpc_types::JsonRpcCall;

pub(crate) use evm::ETH_CALL;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContractCall {
    pub(crate) address: String,
    pub(crate) identifier: String,
}

#[derive(Clone, Copy)]
pub(crate) enum ContractRequest<'a> {
    JsonRpc(&'a JsonRpcCall),
    Http { path: &'a str, method: &'a str, body: &'a [u8] },
}

trait ContractCallDecoder {
    fn decode_contract_calls(&self, request: ContractRequest<'_>) -> Option<Vec<ContractCall>>;
}

pub(crate) fn decode_contract_calls(chain_type: &ChainType, request: ContractRequest<'_>) -> Option<Vec<ContractCall>> {
    match chain_type {
        ChainType::Ethereum => evm::EvmContractCallDecoder.decode_contract_calls(request),
        ChainType::Sui => sui::SuiContractCallDecoder.decode_contract_calls(request),
        _ => None,
    }
}
