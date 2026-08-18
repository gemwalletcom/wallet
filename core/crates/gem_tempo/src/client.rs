use std::error::Error;
use std::str::FromStr;

use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_client::Client;
use gem_evm::rpc::EthereumClient;

use crate::contracts::{ITIP20, ITempoFeeManager};
use crate::fee::FEE_MANAGER_ADDRESS;

pub async fn get_user_fee_token<C: Client + Clone>(client: &EthereumClient<C>, address: &str) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let call_data = ITempoFeeManager::userTokensCall {
        user: Address::from_str(address)?,
    }
    .abi_encode();
    let result = client.eth_call(FEE_MANAGER_ADDRESS, &call_data).await?;
    let token = ITempoFeeManager::userTokensCall::abi_decode_returns(&result)?;
    Ok((!token.is_zero()).then(|| token.to_checksum(None)))
}

pub async fn get_tip20_currency<C: Client + Clone>(client: &EthereumClient<C>, token_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let call_data = ITIP20::currencyCall {}.abi_encode();
    let result = client.eth_call(token_id, &call_data).await?;
    Ok(ITIP20::currencyCall::abi_decode_returns(&result)?)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::testkit::{TEMPO_TEST_ADDRESS, TEMPO_TEST_CBBTC_TOKEN, create_tempo_test_client};
    use primitives::asset_constants::TEMPO_USDC_TOKEN_ID;

    #[tokio::test]
    async fn test_tip20_currency() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_tempo_test_client();

        assert_eq!(get_tip20_currency(&client, TEMPO_USDC_TOKEN_ID).await?, "USD");
        assert_eq!(get_tip20_currency(&client, TEMPO_TEST_CBBTC_TOKEN).await?, "BTC");
        assert_eq!(get_user_fee_token(&client, TEMPO_TEST_ADDRESS).await?, None);

        Ok(())
    }
}
