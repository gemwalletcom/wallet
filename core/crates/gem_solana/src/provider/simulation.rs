use std::collections::HashSet;
use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainSimulation, ChainToken};
use futures::future::join_all;
use gem_client::Client;
use gem_encoding::decode_base64;
use primitives::{Asset, SimulationBalanceChange, SimulationInput, SimulationResult};
use solana_primitives::VersionedTransaction;

use crate::provider::simulation_mapper::map_simulation_result;
use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainSimulation for SolanaClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let bytes = decode_base64(&input.encoded_transaction)?;
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).map_err(|err| format!("parse transaction: {err}"))?;
        let account_keys: Vec<String> = transaction.account_keys().iter().map(|key| key.to_string()).collect();
        let signer_addresses: HashSet<String> = transaction
            .account_keys()
            .iter()
            .take(transaction.num_required_signatures() as usize)
            .map(|key| key.to_string())
            .collect();

        let simulation = self.simulate_encoded_transaction(&input.encoded_transaction).await?;
        let mut result = map_simulation_result(&account_keys, &signer_addresses, simulation);

        let assets = self.get_balance_change_assets(&result.balance_changes).await;
        for (change, (name, symbol)) in result.balance_changes.iter_mut().zip(assets) {
            change.name = name;
            change.symbol = symbol;
        }
        Ok(result)
    }
}

impl<C: Client + Clone> SolanaClient<C> {
    async fn get_balance_change_assets(&self, changes: &[SimulationBalanceChange]) -> Vec<(Option<String>, Option<String>)> {
        join_all(changes.iter().map(|change| async move {
            match &change.asset_id.token_id {
                None => {
                    let asset = Asset::from_chain(self.get_chain());
                    (Some(asset.name), Some(asset.symbol))
                }
                Some(mint) => match self.get_token_data(mint.clone()).await {
                    Ok(asset) => (Some(asset.name), Some(asset.symbol)),
                    Err(_) => (None, None),
                },
            }
        }))
        .await
    }
}
