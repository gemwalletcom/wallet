use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest, TransactionsResult};
use gem_evm::address::ethereum_address_checksum;
use primitives::{Transaction, TransactionIdRequest};
use std::collections::HashSet;
use std::error::Error;

use gem_client::Client;

use crate::{
    models::{order::UserFill, spot::SpotMeta, transaction_id::HyperCoreSignedOrderId},
    provider::{
        transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS,
        transactions_mapper::{map_signed_order_fills, map_user_fills},
    },
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransaction for HyperCoreClient<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(order) = HyperCoreSignedOrderId::parse(&request.hash) else {
            return Ok(None);
        };
        let role = self.get_user_role(&order.signer).await?;
        let owner = ethereum_address_checksum(&role.owner(&order.signer))?;
        let fills = get_user_fills_since(self, &owner, order.nonce.saturating_sub(ACTION_HISTORY_QUERY_LOOKBACK_MS) as i64).await?;
        let fills = map_signed_order_fills(&order, fills);
        let spot_meta = load_spot_meta_if_needed(self, &fills).await?;
        Ok(map_user_fills(&owner, fills, spot_meta.as_ref()).into_iter().next())
    }
}

#[async_trait]
impl<C: Client> ChainBlockTransactions for HyperCoreClient<C> {}

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let start_time = request.from_timestamp.map(|ts| ts as i64 * 1000).unwrap_or(0);
        let fills = get_user_fills_since(self, &request.address, start_time).await?;
        let spot_meta = load_spot_meta_if_needed(self, &fills).await?;
        let transactions = map_user_fills(&request.address, fills, spot_meta.as_ref());

        let transactions = match request.asset_id {
            Some(asset_id) => transactions.into_iter().filter(|transaction| transaction.asset_ids().contains(&asset_id)).collect(),
            None => transactions,
        };
        Ok(TransactionsResult::Transactions(transactions))
    }
}

const USER_FILLS_PAGE_SIZE: usize = 2000;

async fn get_user_fills_since<C: Client>(client: &HyperCoreClient<C>, address: &str, start_time: i64) -> Result<Vec<UserFill>, Box<dyn Error + Sync + Send>> {
    let mut fills: Vec<UserFill> = Vec::new();
    let mut start_time = start_time;
    loop {
        let page = client.get_user_fills_by_time(address, start_time).await?;
        let is_full_page = page.len() >= USER_FILLS_PAGE_SIZE;
        let next_start_time = page.last().map(|fill| fill.time as i64);
        let tids = fills.iter().map(|fill| fill.tid).collect::<HashSet<_>>();
        fills.extend(page.into_iter().filter(|fill| !tids.contains(&fill.tid)));
        match next_start_time {
            Some(next_start_time) if is_full_page && next_start_time > start_time => start_time = next_start_time,
            _ => return Ok(fills),
        }
    }
}

async fn load_spot_meta_if_needed<C: Client>(client: &HyperCoreClient<C>, fills: &[UserFill]) -> Result<Option<SpotMeta>, Box<dyn Error + Sync + Send>> {
    if fills.iter().any(|fill| fill.coin.starts_with('@')) {
        return Ok(Some(client.get_spot_meta().await?));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;
    use primitives::Chain;
    use serde_json::{Value, json};

    fn fill(tid: u64, time: u64) -> Value {
        json!({"coin": "HYPE", "hash": format!("0x{tid}"), "oid": tid, "tid": tid, "sz": "1", "closedPnl": "0", "fee": "0", "px": "1", "dir": "Open Long", "time": time})
    }

    #[tokio::test]
    async fn test_get_user_fills_since_pages_by_time() {
        let client = HyperCoreClient::mock_with_client(MockClient::new().with_post(|_, body| {
            let start_time = serde_json::from_slice::<Value>(body).unwrap()["startTime"].as_u64().unwrap();
            let fills: Vec<Value> = match start_time {
                0 => (1..=USER_FILLS_PAGE_SIZE as u64).map(|tid| fill(tid, tid)).collect(),
                2000 => (2000..2500).map(|tid| fill(tid, tid)).collect(),
                _ => vec![],
            };
            Ok(serde_json::to_vec(&fills).unwrap())
        }));

        let fills = get_user_fills_since(&client, "0xuser", 0).await.unwrap();

        assert_eq!(fills.len(), 2499);
        assert_eq!(fills.last().unwrap().tid, 2499);
    }

    fn signed_order_client(role: Value) -> HyperCoreClient<MockClient> {
        HyperCoreClient::mock_with_client(MockClient::new().with_post(move |_, body| {
            let payload = serde_json::from_slice::<Value>(body).unwrap();
            let response = match payload["type"].as_str().unwrap() {
                "userRole" => role.clone(),
                "userFillsByTime" => {
                    assert_eq!(payload["user"], "0xF5421eFCe6a6FEde2AB38bAA02e9E87BF16ef534");
                    assert_eq!(payload["startTime"], 1790829867053u64);
                    json!([fill(7, 1790829872100), fill(8, 1790829880000)])
                }
                other => panic!("unexpected request {other}"),
            };
            Ok(serde_json::to_vec(&response).unwrap())
        }))
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash_loads_agent_order_fills() {
        let client = signed_order_client(json!({"role": "agent", "data": {"user": "0xf5421efce6a6fede2ab38baa02e9e87bf16ef534"}}));

        for (id, hash) in [
            ("signedOrder:0xd864a8c0ba6f7a3008ddf148e1194813ba3842d2:1790829872053:8", "0x8"),
            ("signedOrder:0xd864a8c0ba6f7a3008ddf148e1194813ba3842d2:1790829872053", "0x7"),
        ] {
            let transaction = client.get_transaction_by_hash(TransactionIdRequest::new(Chain::HyperCore, id.to_string(), None)).await.unwrap().unwrap();

            assert_eq!(transaction.hash(), hash);
            assert_eq!(transaction.from, "0xF5421eFCe6a6FEde2AB38bAA02e9E87BF16ef534");
        }
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash_loads_user_order_fills() {
        let client = signed_order_client(json!({"role": "user"}));
        let request = TransactionIdRequest::new(Chain::HyperCore, "signedOrder:0xf5421efce6a6fede2ab38baa02e9e87bf16ef534:1790829872053:7".to_string(), None);

        assert_eq!(client.get_transaction_by_hash(request).await.unwrap().unwrap().hash(), "0x7");
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash_skips_unsigned_ids() {
        let client = HyperCoreClient::mock_with_client(MockClient::new());

        for id in ["order:561960681274", "action:order:1790829872053", "action:123"] {
            assert_eq!(client.get_transaction_by_hash(TransactionIdRequest::new(Chain::HyperCore, id.to_string(), None)).await.unwrap(), None);
        }
    }
}
