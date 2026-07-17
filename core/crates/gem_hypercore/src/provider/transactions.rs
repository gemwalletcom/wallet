use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;

use crate::{
    models::{order::UserFill, spot::SpotMeta},
    provider::transactions_mapper::map_user_fills,
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let start_time = request.from_timestamp.map(|ts| ts as i64 * 1000).unwrap_or(0);
        let fills = self.get_user_fills_by_time(&request.address, start_time).await?;
        let spot_meta = load_spot_meta_if_needed(self, &fills).await?;
        let transactions = map_user_fills(&request.address, fills, spot_meta.as_ref());

        let transactions = match request.asset_id {
            Some(asset_id) => transactions.into_iter().filter(|transaction| transaction.asset_ids().contains(&asset_id)).collect(),
            None => transactions,
        };
        Ok(TransactionsResult::Transactions(transactions))
    }
}

async fn load_spot_meta_if_needed<C: Client>(client: &HyperCoreClient<C>, fills: &[UserFill]) -> Result<Option<SpotMeta>, Box<dyn Error + Sync + Send>> {
    if fills.iter().any(|fill| fill.coin.starts_with('@')) {
        return Ok(Some(client.get_spot_meta().await?));
    }
    Ok(None)
}
