use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

const GET_ASSET_TRANSFERS: &str = "alchemy_getAssetTransfers";
const GET_TOKEN_BALANCES: &str = "alchemy_getTokenBalances";
const TRANSFER_CATEGORIES: [&str; 4] = ["external", "erc20", "erc721", "erc1155"];

#[derive(Clone, Copy, Debug)]
pub enum TransferDirection {
    From,
    To,
}

impl TransferDirection {
    fn field(self) -> &'static str {
        match self {
            Self::From => "fromAddress",
            Self::To => "toAddress",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum AlchemyRpc {
    GetAssetTransfers { direction: TransferDirection, address: String, limit: usize },
    GetTokenBalances { address: String },
}

impl ToJsonRpcRequest for AlchemyRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::GetAssetTransfers { .. } => GET_ASSET_TRANSFERS,
            Self::GetTokenBalances { .. } => GET_TOKEN_BALANCES,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::GetAssetTransfers { direction, address, limit } => json!([{
                "category": TRANSFER_CATEGORIES,
                "excludeZeroValue": false,
                "maxCount": format!("0x{limit:x}"),
                "order": "desc",
                direction.field(): address,
            }]),
            Self::GetTokenBalances { address } => json!([address, "erc20"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_asset_transfers_request() {
        let request = AlchemyRpc::GetAssetTransfers {
            direction: TransferDirection::To,
            address: "0x1234".into(),
            limit: 2,
        }
        .to_jsonrpc_request(7);

        assert_eq!(request.method, GET_ASSET_TRANSFERS);
        assert_eq!(
            request.params,
            json!([{
                "category": ["external", "erc20", "erc721", "erc1155"],
                "excludeZeroValue": false,
                "maxCount": "0x2",
                "order": "desc",
                "toAddress": "0x1234",
            }])
        );
    }

    #[test]
    fn builds_token_balances_request() {
        let request = AlchemyRpc::GetTokenBalances { address: "0x1234".into() }.to_jsonrpc_request(7);

        assert_eq!(request.method, GET_TOKEN_BALANCES);
        assert_eq!(request.params, json!(["0x1234", "erc20"]));
    }
}
