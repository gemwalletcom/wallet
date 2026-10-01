use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest, TransactionsResult};
use std::collections::HashSet;
use std::error::Error;

use gem_client::Client;

use crate::{
    models::{order::UserFill, spot::SpotMeta},
    provider::transactions_mapper::map_user_fills,
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransaction for HyperCoreClient<C> {}

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
}
