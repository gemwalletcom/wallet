use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

use crate::method;

#[derive(Clone, Debug)]
pub(super) enum AlchemyRpc {
    GetAssetTransfers(Value),
    GetTokenBalances(String),
}

impl ToJsonRpcRequest for AlchemyRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::GetAssetTransfers(_) => method::ALCHEMY_GET_ASSET_TRANSFERS,
            Self::GetTokenBalances(_) => method::ALCHEMY_GET_TOKEN_BALANCES,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::GetAssetTransfers(request) => json!([request]),
            Self::GetTokenBalances(address) => json!([address, "erc20"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_token_balances_request() {
        let request = AlchemyRpc::GetTokenBalances("0x1234".into()).to_jsonrpc_request(7);

        assert_eq!(request.method, method::ALCHEMY_GET_TOKEN_BALANCES);
        assert_eq!(request.params, json!(["0x1234", "erc20"]));
    }
}
