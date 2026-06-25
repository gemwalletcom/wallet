use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::error::Error;

use crate::models::rpc::{AccountInfo, AccountInfoResult, AccountLedger, AccountObjects, FeesResult, Ledger, LedgerCurrent, LedgerData, TransactionBroadcast, TransactionStatus};

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits};
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use gem_jsonrpc::types::{ERROR_CLIENT_ERROR, JsonRpcError};
use primitives::Chain;

#[derive(Clone, Debug)]
pub struct XRPClient<C: Client + Clone> {
    client: GenericJsonRpcClient<C>,
    pub chain: Chain,
}

impl<C: Client + Clone> XRPClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Xrp }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub fn get_client(&self) -> &GenericJsonRpcClient<C> {
        &self.client
    }

    async fn call<T>(&self, method: &str, params: impl Into<Value>) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned + Send,
    {
        let result: Value = self.client.call(method, params).await?;
        deserialize_result(result)
    }

    pub async fn get_account_info(&self, address: &str) -> Result<Option<AccountInfo>, Box<dyn Error + Send + Sync>> {
        let result = self.get_account_info_full(address).await?;
        Ok(result.account_data)
    }

    pub async fn get_account_info_full(&self, address: &str) -> Result<AccountInfoResult, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "account": address,
                "ledger_index": "current"
            }
        ]);

        self.call("account_info", params).await
    }

    pub async fn get_ledger_current(&self) -> Result<LedgerCurrent, Box<dyn Error + Send + Sync>> {
        let params = json!([{}]);
        self.call("ledger_current", params).await
    }

    pub async fn get_fees(&self) -> Result<FeesResult, Box<dyn Error + Send + Sync>> {
        let params = json!([{}]);
        self.call("fee", params).await
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<TransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "tx_blob": data,
                "fail_hard": true
            }
        ]);

        self.call("submit", params).await
    }

    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, Box<dyn Error + Send + Sync>> {
        self.get_transaction(transaction_id).await
    }

    pub(crate) async fn get_transaction<T>(&self, transaction_id: &str) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: DeserializeOwned + Send,
    {
        let params = json!([
            {
                "transaction": transaction_id
            }
        ]);
        self.call("tx", params).await
    }

    pub async fn get_account_objects(&self, address: &str) -> Result<AccountObjects, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "account": address,
                "type": "state",
                "ledger_index": "validated"
            }
        ]);

        self.call("account_objects", params).await
    }

    pub async fn get_block_transactions(&self, block_number: u64) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "ledger_index": block_number,
                "transactions": true,
                "expand": true
            }
        ]);

        let result: LedgerData = self.call("ledger", params).await?;
        Ok(result.ledger)
    }

    pub async fn get_account_transactions(&self, address: String, limit: usize) -> Result<AccountLedger, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "account": address,
                "limit": limit,
                "ledger_index_max": -1,
                "ledger_index_min": -1
            }
        ]);

        self.call("account_tx", params).await
    }
}

fn deserialize_result<T>(result: Value) -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: DeserializeOwned,
{
    if let Some(error) = map_error_result(&result) {
        return Err(Box::new(error));
    }

    Ok(serde_json::from_value(result)?)
}

fn map_error_result(result: &Value) -> Option<JsonRpcError> {
    if result.get("status").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let code = result
        .get("error_code")
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .unwrap_or(ERROR_CLIENT_ERROR);
    let error = result.get("error").and_then(Value::as_str);
    let error_message = result.get("error_message").and_then(Value::as_str);
    let message = match (error, error_message) {
        (Some(error), Some(error_message)) if error != error_message => format!("{error}: {error_message}"),
        (Some(error), _) => error.to_string(),
        (None, Some(error_message)) => error_message.to_string(),
        (None, None) => "XRP RPC error".to_string(),
    };

    Some(JsonRpcError { code, message })
}

impl<C: Client + Clone> ChainStaking for XRPClient<C> {}

impl<C: Client + Clone> ChainPerpetual for XRPClient<C> {}

impl<C: Client + Clone> ChainAddressStatus for XRPClient<C> {}

impl<C: Client + Clone> chain_traits::ChainAccount for XRPClient<C> {}

impl<C: Client + Clone> ChainSimulation for XRPClient<C> {}

impl<C: Client + Clone> ChainTraits for XRPClient<C> {}

impl<C: Client + Clone> ChainProvider for XRPClient<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_result() {
        let ledger = deserialize_result::<LedgerCurrent>(json!({
            "ledger_current_index": 80123456,
            "status": "success"
        }))
        .unwrap();
        assert_eq!(ledger.ledger_current_index, 80123456);

        let error = deserialize_result::<LedgerCurrent>(json!({
            "error": "amendmentBlocked",
            "error_code": 14,
            "error_message": "Amendment blocked, need upgrade.",
            "request": {
                "command": "ledger_current"
            },
            "status": "error"
        }))
        .unwrap_err();
        assert_eq!(error.to_string(), "amendmentBlocked: Amendment blocked, need upgrade. (14)");
    }
}
