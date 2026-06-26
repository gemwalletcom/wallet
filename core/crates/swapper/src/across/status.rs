use std::{str::FromStr, sync::Arc};

use alloy_primitives::{Address, B256, U256, hex::encode_prefixed as HexEncode, keccak256};
use alloy_sol_types::SolValue;
use gem_evm::{across::contracts::V3SpokePoolInterface, across::deployment::AcrossDeployment};
use gem_tron::rpc::constants::{RECEIPT_FAILED, RECEIPT_OUT_OF_ENERGY};
use primitives::{Chain, TransactionSwapMetadata, swap::SwapStatus};

use super::{
    asset::supported_asset_for_token,
    deposit::{FillStatusRelayData, ParsedDeposit, parse_deposit_from_logs, parse_deposit_from_tron_receipt},
};
use crate::{
    SwapResult, SwapperError, SwapperProvider,
    alien::RpcProvider,
    client_factory::{create_eth_client, create_tron_client},
};

const ACROSS_FILL_STATUS_FILLED: u8 = 2;

pub(super) async fn get_swap_result(rpc_provider: Arc<dyn RpcProvider>, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
    let deposit = match source_deposit(rpc_provider.clone(), chain, transaction_hash).await? {
        SourceDeposit::Deposit(deposit) => deposit,
        SourceDeposit::Pending => {
            return Ok(SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
            });
        }
        SourceDeposit::Failed => {
            return Ok(SwapResult {
                status: SwapStatus::Failed,
                metadata: None,
            });
        }
    };

    if !is_filled(rpc_provider, &deposit).await? {
        return Ok(SwapResult {
            status: SwapStatus::Pending,
            metadata: None,
        });
    }

    Ok(SwapResult {
        status: SwapStatus::Completed,
        metadata: swap_metadata(&deposit),
    })
}

enum SourceDeposit {
    Pending,
    Failed,
    Deposit(Box<ParsedDeposit>),
}

impl SourceDeposit {
    fn from_parsed(deposit: Option<ParsedDeposit>) -> Self {
        deposit.map(Box::new).map(SourceDeposit::Deposit).unwrap_or(SourceDeposit::Pending)
    }
}

fn across_chain_id(chain: Chain) -> Result<u64, SwapperError> {
    Ok(u64::from(AcrossDeployment::deployment_by_chain(&chain).ok_or(SwapperError::NotSupportedChain)?.chain_id))
}

async fn source_deposit(rpc_provider: Arc<dyn RpcProvider>, chain: Chain, transaction_hash: &str) -> Result<SourceDeposit, SwapperError> {
    let origin_chain_id = across_chain_id(chain)?;

    if chain == Chain::Tron {
        let Some(receipt) = create_tron_client(rpc_provider)?
            .get_transaction_reciept(transaction_hash.to_string())
            .await
            .map_err(SwapperError::transaction_error)?
        else {
            return Ok(SourceDeposit::Pending);
        };

        if receipt
            .receipt
            .result
            .as_deref()
            .is_some_and(|result| result == RECEIPT_FAILED || result == RECEIPT_OUT_OF_ENERGY)
        {
            return Ok(SourceDeposit::Failed);
        }

        return Ok(SourceDeposit::from_parsed(parse_deposit_from_tron_receipt(&receipt, origin_chain_id)?));
    }

    let Some(receipt) = create_eth_client(rpc_provider, chain)?
        .get_transaction_receipt(transaction_hash)
        .await
        .map_err(SwapperError::from)?
    else {
        return Ok(SourceDeposit::Pending);
    };

    Ok(SourceDeposit::from_parsed(parse_deposit_from_logs(&receipt.logs, origin_chain_id)?))
}

async fn is_filled(rpc_provider: Arc<dyn RpcProvider>, deposit: &ParsedDeposit) -> Result<bool, SwapperError> {
    let destination_chain = Chain::from_chain_id(deposit.destination_chain_id).ok_or(SwapperError::NotSupportedChain)?;
    let deployment = AcrossDeployment::deployment_by_chain(&destination_chain).ok_or(SwapperError::NotSupportedChain)?;
    let relay_hash = relay_hash(&deposit.relay_data, deposit.destination_chain_id);

    let client = create_eth_client(rpc_provider, destination_chain)?;
    let spoke_pool = Address::from_str(deployment.spoke_pool).map_err(SwapperError::transaction_error)?;
    let fill_status: u8 = client
        .call_contract(spoke_pool, V3SpokePoolInterface::fillStatusesCall { relayHash: relay_hash })
        .await
        .map_err(SwapperError::transaction_error)?;

    Ok(fill_status == ACROSS_FILL_STATUS_FILLED)
}

fn relay_hash(relay_data: &FillStatusRelayData, destination_chain_id: u64) -> B256 {
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

pub(in crate::across) fn swap_metadata(deposit: &ParsedDeposit) -> Option<TransactionSwapMetadata> {
    let relay_data = &deposit.relay_data;
    let origin_chain = Chain::from_chain_id(relay_data.origin_chain_id.to::<u64>())?;
    let from_asset = supported_asset_for_token(origin_chain, &word_address(relay_data.input_token))?;
    let to_chain = Chain::from_chain_id(deposit.destination_chain_id)?;
    let to_asset = supported_asset_for_token(to_chain, &word_address(relay_data.output_token))?;
    Some(TransactionSwapMetadata {
        from_asset,
        from_value: relay_data.input_amount.to_string(),
        to_asset,
        to_value: relay_data.output_amount.to_string(),
        provider: Some(SwapperProvider::Across.as_ref().to_string()),
    })
}

fn word_address(word: B256) -> String {
    HexEncode(Address::from_word(word))
}
