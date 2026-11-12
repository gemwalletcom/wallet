use super::{
    constants::evm_gas_limit,
    model::{SignatureData, TransactionData},
};
use crate::{
    SwapperError,
    alien::RpcProvider,
    approval::{check_approval_erc20, get_swap_gas_limit_with_approval},
};
use alloy_primitives::U256;
use gem_encoding::{decode_base58, encode_base64};
use gem_evm::provider::preload_mapper::calculate_gas_limit_with_increase;
use gem_solana::{Pubkey, VersionedTransaction};
use num_bigint::BigInt;
use num_bigint::BigUint;
use primitives::{
    Chain, ChainType,
    swap::{ApprovalData, QuoteAsset, SwapQuoteData},
};
use std::{str::FromStr, sync::Arc};

pub(super) async fn build_swap_quote_data(transaction_data: &TransactionData, from_asset: &QuoteAsset, from_value: &str, chain: Chain, owner: &str, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapQuoteData, SwapperError> {
    match chain.chain_type() {
        ChainType::Ethereum => build_evm_quote_data(transaction_data, from_asset, from_value, chain, owner, rpc_provider).await,
        ChainType::Solana => build_solana_quote_data(transaction_data, owner),
        _ => Err(SwapperError::NotSupportedChain),
    }
}

pub(super) async fn build_evm_quote_data(transaction_data: &TransactionData, from_asset: &QuoteAsset, from_value: &str, chain: Chain, owner: &str, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapQuoteData, SwapperError> {
    let approval = build_evm_approval(from_asset, transaction_data, from_value, chain, owner, rpc_provider).await?;
    let gas_limit = get_swap_gas_limit_with_approval(&approval, buffered_gas_limit(&transaction_data.gas), evm_gas_limit(chain));
    Ok(SwapQuoteData::new_contract(
        transaction_data.to.clone(),
        transaction_data.get_value().ok_or_else(|| SwapperError::ComputeQuoteError("invalid OKX transaction value".to_string()))?,
        transaction_data.data.clone(),
        approval,
        gas_limit,
    ))
}

pub(super) fn build_solana_quote_data(transaction_data: &TransactionData, owner: &str) -> Result<SwapQuoteData, SwapperError> {
    let bytes = decode_base58(&transaction_data.data).map_err(|error| SwapperError::TransactionError(format!("invalid swap transaction data: {error}")))?;
    let mut transaction = VersionedTransaction::deserialize_with_version(&bytes).map_err(SwapperError::transaction_error)?;
    if transaction.num_required_signatures() != 1 {
        return Err(SwapperError::transaction_error("OKX transaction requires more than one signer"));
    }
    transaction.clear_signature(&Pubkey::from_base58(owner).map_err(SwapperError::transaction_error)?);
    let data = encode_base64(&transaction.serialize().map_err(SwapperError::transaction_error)?);
    let gas_limit = transaction.get_compute_unit_limit().map(|limit| limit.to_string());
    Ok(SwapQuoteData::new_contract(transaction_data.to.clone(), BigUint::from(0u64), data, None, gas_limit))
}

fn buffered_gas_limit(gas: &str) -> Option<String> {
    gas.parse::<BigInt>().ok().filter(|value| *value > BigInt::from(0)).map(|value| calculate_gas_limit_with_increase(value).to_string())
}

async fn build_evm_approval(from_asset: &QuoteAsset, transaction_data: &TransactionData, from_value: &str, chain: Chain, owner: &str, rpc_provider: Arc<dyn RpcProvider>) -> Result<Option<ApprovalData>, SwapperError> {
    let Some(token) = from_asset.asset_id().token_id else {
        return Ok(None);
    };
    let spender = get_spender(transaction_data.signature_data.as_deref()).ok_or(SwapperError::InvalidRoute)?;
    let amount = U256::from_str(from_value)?;
    check_approval_erc20(owner.to_string(), token, spender, amount, rpc_provider, &chain).await
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
    use gem_encoding::encode_base58;

    #[test]
    fn test_solana_quote_data_requires_the_wallet_as_the_only_signer() {
        let response: serde_json::Value = serde_json::from_str(include_str!("testdata/swap_sol_to_usdc.json")).unwrap();
        let encoded = response["data"][0]["tx"]["data"].as_str().unwrap();
        let mut transaction = VersionedTransaction::deserialize_with_version(&decode_base58(encoded).unwrap()).unwrap();
        let owner = transaction.account_keys()[0].to_base58();
        let single_signer = TransactionData {
            data: encoded.to_string(),
            to: String::new(),
            value: String::new(),
            gas: String::new(),
            signature_data: None,
        };
        assert!(build_solana_quote_data(&single_signer, &owner).is_ok());

        transaction.message_mut().header.num_required_signatures = 2;
        let two_signers = TransactionData {
            data: encode_base58(transaction.serialize().unwrap()),
            ..single_signer
        };
        assert!(
            matches!(build_solana_quote_data(&two_signers, &owner), Err(SwapperError::TransactionError(_))),
            "a second signer slot could carry a stranger's pre-signed instruction"
        );
    }

    #[test]
    fn test_buffered_gas_limit() {
        assert_eq!(buffered_gas_limit("200000").as_deref(), Some("300000"));
        assert_eq!(buffered_gas_limit(""), None);
        assert_eq!(buffered_gas_limit("0"), None);
    }
}
