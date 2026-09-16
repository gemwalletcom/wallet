use primitives::{WalletConnectionMethods, serde_name};
use serde_json::Value;

use crate::wallet_connect_pay::model::WalletRpcAction;

impl WalletRpcAction {
    pub fn mock(method: WalletConnectionMethods, chain_id: &str, params: Value) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            method: serde_name(&method).unwrap(),
            params,
        }
    }
}
