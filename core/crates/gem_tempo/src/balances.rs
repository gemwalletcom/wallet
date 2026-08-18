use std::error::Error;

use gem_client::Client;
use gem_evm::rpc::EthereumClient;
use primitives::{AssetBalance, asset_constants::TEMPO_PATHUSD_TOKEN_ID};

pub async fn get_balance_coin<C: Client + Clone>(client: &EthereumClient<C>, address: &str) -> Result<AssetBalance, Box<dyn Error + Send + Sync>> {
    let balance_hex = client
        .batch_token_balance_calls(address, &[TEMPO_PATHUSD_TOKEN_ID.to_string()])
        .await?
        .into_iter()
        .next()
        .ok_or("Missing native balance result")?;

    gem_evm::provider::balances_mapper::map_balance_coin(balance_hex, client.get_chain())
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::testkit::{TEMPO_TEST_ADDRESS, create_tempo_test_client};
    use num_bigint::BigUint;
    use primitives::Chain;

    #[tokio::test]
    async fn test_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_tempo_test_client();
        let balance = get_balance_coin(&client, TEMPO_TEST_ADDRESS).await?;

        println!("Tempo pathUSD Balance: {:?}", balance.balance.available);

        assert_eq!(balance.asset_id.chain, Chain::Tempo);
        assert!(balance.balance.available > BigUint::from(0u32));

        Ok(())
    }
}
