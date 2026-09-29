use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

use super::model::TransferCategory;

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
    AssetTransfers {
        direction: TransferDirection,
        address: String,
        limit: usize,
    },
    TransfersTo {
        address: String,
        categories: Vec<TransferCategory>,
        from_block: u64,
        page_key: Option<String>,
        limit: usize,
    },
    TokenBalances {
        address: String,
    },
}

impl ToJsonRpcRequest for AlchemyRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::AssetTransfers { .. } | Self::TransfersTo { .. } => GET_ASSET_TRANSFERS,
            Self::TokenBalances { .. } => GET_TOKEN_BALANCES,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::AssetTransfers { direction, address, limit } => json!([{
                "category": TRANSFER_CATEGORIES,
                "excludeZeroValue": false,
                "maxCount": format!("0x{limit:x}"),
                "order": "desc",
                direction.field(): address,
            }]),
            Self::TransfersTo {
                address,
                categories,
                from_block,
                page_key,
                limit,
            } => {
                let mut params = json!({
                    "category": categories,
                    "excludeZeroValue": true,
                    "fromBlock": format!("0x{from_block:x}"),
                    "maxCount": format!("0x{limit:x}"),
                    "order": "asc",
                    "toAddress": address,
                });
                if let Some(page_key) = page_key {
                    params["pageKey"] = json!(page_key);
                }
                json!([params])
            }
            Self::TokenBalances { address } => json!([address, "erc20"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_asset_transfers_request() {
        let request = AlchemyRpc::AssetTransfers {
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
        let request = AlchemyRpc::TokenBalances { address: "0x1234".into() }.to_jsonrpc_request(7);

        assert_eq!(request.method, GET_TOKEN_BALANCES);
        assert_eq!(request.params, json!(["0x1234", "erc20"]));
    }
}
