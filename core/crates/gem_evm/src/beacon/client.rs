use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::{BeaconResponse, PendingDeposit, StateRoot, ValidatorEntry, ValidatorQueue};

const FINALIZED_STATE_ROOT_PATH: &str = "/eth/v1/beacon/states/finalized/root";

pub async fn get_validator_queue(url: &str) -> Result<ValidatorQueue, Box<dyn Error + Send + Sync>> {
    let client = ReqwestClient::new(url.to_string(), gem_client::reqwest_client());
    let response: BeaconResponse<StateRoot> = client.get(FINALIZED_STATE_ROOT_PATH).await?;
    let state_path = format!("/eth/v1/beacon/states/{}", response.verified_data()?.root);
    let pending_deposits_path = format!("{state_path}/pending_deposits");
    let exiting_validators_path = format!("{state_path}/validators?status=active_exiting&status=active_slashed");

    let pending_deposits_request = client.get::<BeaconResponse<Vec<PendingDeposit>>>(&pending_deposits_path);
    let exiting_validators_request = client.get::<BeaconResponse<Vec<ValidatorEntry>>>(&exiting_validators_path);
    let (pending_deposits, exiting_validators) = futures::try_join!(pending_deposits_request, exiting_validators_request)?;

    Ok(ValidatorQueue::new(pending_deposits.verified_data()?, exiting_validators.verified_data()?))
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;

    const PUBLICNODE_BEACON_API_BASE_URL: &str = "https://ethereum-beacon-api.publicnode.com";

    #[tokio::test]
    async fn test_publicnode_get_validator_queue() -> Result<(), Box<dyn Error + Send + Sync>> {
        let queue = get_validator_queue(PUBLICNODE_BEACON_API_BASE_URL).await?;
        let estimate = queue.estimated_wait_times()?;

        println!("Entry queue: {} Gwei, wait: {:?}", queue.entry_queue_balance, estimate.entry_wait_time);
        println!("Exit queue: {} Gwei, wait: {:?}", queue.exit_queue_balance, estimate.exit_wait_time);

        Ok(())
    }
}
