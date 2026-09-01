use super::{
    constants::{TRON_DEX_TOKEN_APPROVE_ADDRESS, evm_gas_limit},
    model::{SignatureData, TransactionData},
};
use crate::{
    SwapperError,
    alien::RpcProvider,
    approval::{check_approval_erc20, check_approval_trc20, get_swap_gas_limit_with_approval},
    models::ApprovalType,
};
use alloy_primitives::U256;
use gem_encoding::encode_base64;
use gem_evm::provider::preload_mapper::calculate_gas_limit_with_increase;
use num_bigint::BigInt;
use num_bigint::BigUint;
use primitives::{
    Chain, ChainType,
    swap::{ApprovalData, QuoteAsset, SwapQuoteData},
};
use std::{str::FromStr, sync::Arc};

pub(super) async fn build_swap_quote_data(
    transaction_data: &TransactionData,
    from_asset: &QuoteAsset,
    from_value: &str,
    chain: Chain,
    owner: &str,
    rpc_provider: Arc<dyn RpcProvider>,
) -> Result<SwapQuoteData, SwapperError> {
    match chain.chain_type() {
        ChainType::Ethereum => build_evm_quote_data(transaction_data, from_asset, from_value, chain, owner, rpc_provider).await,
        ChainType::Solana => build_solana_quote_data(transaction_data),
        ChainType::Tron => build_tron_quote_data(transaction_data, from_asset, from_value, owner, rpc_provider).await,
        _ => Err(SwapperError::NotSupportedChain),
    }
}

pub(super) async fn build_evm_quote_data(
    transaction_data: &TransactionData,
    from_asset: &QuoteAsset,
    from_value: &str,
    chain: Chain,
    owner: &str,
    rpc_provider: Arc<dyn RpcProvider>,
) -> Result<SwapQuoteData, SwapperError> {
    let approval = build_evm_approval(from_asset, transaction_data, from_value, chain, owner, rpc_provider).await?;
    let gas_limit = get_swap_gas_limit_with_approval(&approval, buffered_gas_limit(&transaction_data.gas), evm_gas_limit(chain));
    Ok(SwapQuoteData::new_contract(
        transaction_data.to.clone(),
        transaction_data
            .get_value()
            .ok_or_else(|| SwapperError::ComputeQuoteError("invalid OKX transaction value".to_string()))?,
        transaction_data.data.clone(),
        approval,
        gas_limit,
    ))
}

pub(super) async fn build_tron_quote_data(
    transaction_data: &TransactionData,
    from_asset: &QuoteAsset,
    from_value: &str,
    owner: &str,
    rpc_provider: Arc<dyn RpcProvider>,
) -> Result<SwapQuoteData, SwapperError> {
    let approval = build_tron_approval(from_asset, from_value, owner, rpc_provider).await?;
    let gas_limit = approval
        .is_some()
        .then(|| transaction_data.gas.clone())
        .filter(|gas| gas.parse::<u64>().is_ok_and(|energy| energy > 0));
    let call_data = transaction_data.data.strip_prefix("0x").unwrap_or(&transaction_data.data).to_string();
    Ok(SwapQuoteData::new_contract(
        transaction_data.to.clone(),
        transaction_data
            .get_value()
            .ok_or_else(|| SwapperError::ComputeQuoteError("invalid OKX transaction value".to_string()))?,
        call_data,
        approval,
        gas_limit,
    ))
}

pub(super) fn build_solana_quote_data(transaction_data: &TransactionData) -> Result<SwapQuoteData, SwapperError> {
    let bytes = bs58::decode(&transaction_data.data)
        .into_vec()
        .map_err(|err| SwapperError::TransactionError(format!("invalid swap transaction data: {err}")))?;
    Ok(SwapQuoteData::new_contract(
        transaction_data.to.clone(),
        BigUint::from(0u64),
        encode_base64(&bytes),
        None,
        None,
    ))
}

fn buffered_gas_limit(gas: &str) -> Option<String> {
    gas.parse::<BigInt>()
        .ok()
        .filter(|value| *value > BigInt::from(0))
        .map(|value| calculate_gas_limit_with_increase(value).to_string())
}

async fn build_tron_approval(from_asset: &QuoteAsset, from_value: &str, owner: &str, rpc_provider: Arc<dyn RpcProvider>) -> Result<Option<ApprovalData>, SwapperError> {
    let Some(token) = from_asset.asset_id().token_id else {
        return Ok(None);
    };
    let amount = U256::from_str(from_value)?;
    match check_approval_trc20(owner.to_string(), token, TRON_DEX_TOKEN_APPROVE_ADDRESS.to_string(), amount, rpc_provider).await? {
        ApprovalType::Approve(data) => Ok(Some(data)),
        _ => Ok(None),
    }
}

async fn build_evm_approval(
    from_asset: &QuoteAsset,
    transaction_data: &TransactionData,
    from_value: &str,
    chain: Chain,
    owner: &str,
    rpc_provider: Arc<dyn RpcProvider>,
) -> Result<Option<ApprovalData>, SwapperError> {
    let Some(token) = from_asset.asset_id().token_id else {
        return Ok(None);
    };
    // Fall back to the transaction target when signature_data omits the approve contract.
    let Some(spender) = get_spender(transaction_data.signature_data.as_deref()).or_else(|| {
        let to = transaction_data.to.clone();
        (!to.is_empty()).then_some(to)
    }) else {
        return Ok(None);
    };
    let amount = U256::from_str(from_value)?;
    match check_approval_erc20(owner.to_string(), token, spender, amount, rpc_provider, &chain).await? {
        ApprovalType::Approve(data) => Ok(Some(data)),
        _ => Ok(None),
    }
}

fn get_spender(signature_data: Option<&[String]>) -> Option<String> {
    signature_data
        .unwrap_or(&[])
        .iter()
        .filter_map(|entry| serde_json::from_str::<SignatureData>(entry).ok())
        .map(|data| data.approve_contract)
        .find(|contract| !contract.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffered_gas_limit() {
        assert_eq!(buffered_gas_limit("200000").as_deref(), Some("300000"));
        assert_eq!(buffered_gas_limit(""), None);
        assert_eq!(buffered_gas_limit("0"), None);
    }
}
