use std::{str::FromStr, sync::Arc};

use alloy_primitives::{Address, B256, U256, keccak256};
use alloy_sol_types::SolValue;
use gem_evm::across::{
    contracts::V3SpokePoolInterface,
    deployment::AcrossDeployment,
    deposit::{Deposit, RelayData, parse_deposit},
};
use primitives::{Chain, DateTime, swap::SwapStatus};

use super::asset::supported_asset_for_token;
use crate::{
    SwapResult, SwapperError,
    alien::RpcProvider,
    client_factory::{create_eth_client, create_tron_client},
};

const ACROSS_FILL_STATUS_UNFILLED: u8 = 0;
const ACROSS_FILL_STATUS_FILLED: u8 = 2;

pub(super) async fn get_swap_result(rpc_provider: Arc<dyn RpcProvider>, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
    let deposit = match source_deposit(rpc_provider.clone(), chain, transaction_hash).await? {
        SourceDeposit::Deposit(deposit) => deposit,
        SourceDeposit::Pending => {
            return Ok(SwapResult::pending());
        }
        SourceDeposit::Failed => {
            return Ok(SwapResult::from(SwapStatus::Failed));
        }
    };

    let fill_status = fill_status(rpc_provider, &deposit).await?;
    Ok(match deposit_swap_status(fill_status, deposit.relay_data.fill_deadline, DateTime::now()) {
        SwapStatus::Completed => SwapResult {
            status: SwapStatus::Completed,
            metadata: deposit.swap_metadata(supported_asset_for_token),
            eta_in_seconds: None,
        },
        status => SwapResult::from(status),
    })
}

fn deposit_swap_status(fill_status: u8, fill_deadline: u32, now: DateTime) -> SwapStatus {
    match fill_status {
        ACROSS_FILL_STATUS_FILLED => SwapStatus::Completed,
        ACROSS_FILL_STATUS_UNFILLED if now.timestamp() > i64::from(fill_deadline) => SwapStatus::Refunded,
        _ => SwapStatus::Pending,
    }
}

enum SourceDeposit {
    Pending,
    Failed,
    Deposit(Box<Deposit>),
}

impl SourceDeposit {
    fn from_parsed(deposit: Option<Deposit>) -> Self {
        deposit.map(Box::new).map(SourceDeposit::Deposit).unwrap_or(SourceDeposit::Pending)
    }
}

async fn source_deposit(rpc_provider: Arc<dyn RpcProvider>, chain: Chain, transaction_hash: &str) -> Result<SourceDeposit, SwapperError> {
    let origin_chain_id = chain.network_id_value().ok_or(SwapperError::NotSupportedChain)?;

    if chain == Chain::Tron {
        let Some(receipt) = create_tron_client(rpc_provider)?.get_transaction_receipt(transaction_hash.to_string()).await.map_err(SwapperError::transaction_error)? else {
            return Ok(SourceDeposit::Pending);
        };

        if receipt.is_failed() {
            return Ok(SourceDeposit::Failed);
        }

        let logs = receipt.log.as_deref().unwrap_or_default();
        let entries = logs.iter().filter_map(|log| Some((log.topics.as_deref()?, log.data.as_deref()?)));
        let deposit = parse_deposit(entries, origin_chain_id).map_err(SwapperError::transaction_error)?;
        return Ok(SourceDeposit::from_parsed(deposit));
    }

    let Some(receipt) = create_eth_client(rpc_provider, chain)?.get_transaction_receipt(transaction_hash).await.map_err(SwapperError::from)? else {
        return Ok(SourceDeposit::Pending);
    };

    let entries = receipt.logs.iter().map(|log| (log.topics.as_slice(), log.data.as_str()));
    let deposit = parse_deposit(entries, origin_chain_id).map_err(SwapperError::transaction_error)?;
    Ok(SourceDeposit::from_parsed(deposit))
}

async fn fill_status(rpc_provider: Arc<dyn RpcProvider>, deposit: &Deposit) -> Result<u8, SwapperError> {
    let destination_chain = Chain::from_chain_id(deposit.destination_chain_id).ok_or(SwapperError::NotSupportedChain)?;
    let deployment = AcrossDeployment::deployment_by_chain(&destination_chain).ok_or(SwapperError::NotSupportedChain)?;
    let relay_hash = relay_hash(&deposit.relay_data, deposit.destination_chain_id);

    let client = create_eth_client(rpc_provider, destination_chain)?;
    let spoke_pool = Address::from_str(deployment.spoke_pool).map_err(SwapperError::transaction_error)?;
    client
        .call_contract(spoke_pool, V3SpokePoolInterface::fillStatusesCall { relayHash: relay_hash })
        .await
        .map_err(SwapperError::transaction_error)
}

fn relay_hash(relay_data: &RelayData, destination_chain_id: u64) -> B256 {
    let relay_data = (
        relay_data.depositor,
        relay_data.recipient,
        relay_data.exclusive_relayer,
        relay_data.input_token,
        relay_data.output_token,
        relay_data.input_amount,
        relay_data.output_amount,
        relay_data.origin_chain_id,
        relay_data.deposit_id,
        relay_data.fill_deadline,
        relay_data.exclusivity_deadline,
        relay_data.message.clone(),
    );
    keccak256((relay_data, U256::from(destination_chain_id)).abi_encode_sequence())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_unfilled_deposit_past_its_fill_deadline_is_refunded() {
        let fill_deadline = 1_763_000_000;
        let before = DateTime::from_timestamp(i64::from(fill_deadline) - 60);
        let after = DateTime::from_timestamp(i64::from(fill_deadline) + 60);

        assert_eq!(deposit_swap_status(ACROSS_FILL_STATUS_UNFILLED, fill_deadline, before), SwapStatus::Pending, "a relayer can still fill it");
        assert_eq!(
            deposit_swap_status(ACROSS_FILL_STATUS_UNFILLED, fill_deadline, after),
            SwapStatus::Refunded,
            "no relayer can fill it, so Across refunds the depositor"
        );
        assert_eq!(deposit_swap_status(1, fill_deadline, after), SwapStatus::Pending, "a requested slow fill still completes");
        assert_eq!(deposit_swap_status(ACROSS_FILL_STATUS_FILLED, fill_deadline, after), SwapStatus::Completed);
    }
}
