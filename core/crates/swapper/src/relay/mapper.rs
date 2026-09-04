use gem_tron::address::TronAddress;
use num_bigint::BigUint;
use primitives::{TransactionSwapMetadata, swap::ApprovalData};
use std::str::FromStr;

use super::{
    asset::map_currency_to_asset_id,
    chain::RelayChain,
    model::{RelayQuoteResponse, RelayRequest},
};
use crate::{
    SwapResult, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, DEFAULT_TRON_SWAP_ENERGY_LIMIT, get_swap_gas_limit_with_approval},
};

pub fn map_evm_quote_data(quote_response: &RelayQuoteResponse, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let evm = quote_response.get_evm_step().ok_or(SwapperError::InvalidRoute)?;
    let gas_limit = get_swap_gas_limit_with_approval(&approval, evm.gas_limit_with_buffer(), DEFAULT_EVM_SWAP_GAS_LIMIT);
    let call_data = evm.data.clone().unwrap_or_default();
    Ok(SwapperQuoteData::new_contract(
        evm.to.clone(),
        BigUint::from_str(&evm.value).map_err(SwapperError::compute_quote_error)?,
        call_data,
        approval,
        gas_limit,
    ))
}

pub fn map_tron_quote_data(quote_response: &RelayQuoteResponse, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let tron = quote_response.get_tron_step().ok_or(SwapperError::InvalidRoute)?;
    let transaction = tron.trigger_smart_contract().ok_or(SwapperError::InvalidRoute)?;
    let contract = TronAddress::parse_hex_or_base58(&transaction.contract_address).map_err(|_| SwapperError::InvalidRoute)?;
    let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_TRON_SWAP_ENERGY_LIMIT);
    Ok(SwapperQuoteData::new_contract(
        contract.to_string(),
        BigUint::from(transaction.call_value.unwrap_or_default()),
        transaction.data.clone(),
        approval,
        gas_limit,
    ))
}

pub fn map_ton_quote_data(quote_response: &RelayQuoteResponse) -> Result<SwapperQuoteData, SwapperError> {
    let ton = quote_response.get_ton_step().ok_or(SwapperError::InvalidRoute)?;
    let [message] = ton.messages.as_slice() else {
        return Err(SwapperError::InvalidRoute);
    };
    Ok(SwapperQuoteData::new_contract(
        message.to.clone(),
        BigUint::from_str(&message.value).map_err(SwapperError::compute_quote_error)?,
        message.body.clone(),
        None,
        None,
    ))
}

pub fn map_swap_result(request: &RelayRequest) -> SwapResult {
    let metadata = request.data.as_ref().and_then(|data| {
        let actual = data.route.as_ref()?.actual.as_ref()?;
        let currency_in = actual.currency_in()?;
        let currency_out = actual.currency_out()?;
        let from_chain = RelayChain::from_chain_id(currency_in.currency.chain_id)?.to_chain();
        let to_chain = RelayChain::from_chain_id(currency_out.currency.chain_id)?.to_chain();
        Some(TransactionSwapMetadata {
            from_asset: map_currency_to_asset_id(from_chain, &currency_in.currency.address),
            from_value: BigUint::from_str(currency_in.amount.as_deref()?).ok()?,
            to_asset: map_currency_to_asset_id(to_chain, &currency_out.currency.address),
            to_value: BigUint::from_str(currency_out.amount.as_deref()?).ok()?,
            provider: Some(SwapperProvider::Relay.as_ref().to_string()),
        })
    });

    SwapResult {
        status: request.status.clone().into_swap_status(),
        metadata,
        eta_in_seconds: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::{RelayQuoteResponse, RelayRequest, RelayRequestsResponse, RelayStatus, Step};
    use primitives::{AssetId, Chain, swap::SwapStatus};

    #[test]
    fn test_map_evm_quote_data() {
        let quote_response = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("swap", "0xrouter", "1000000000000000000", "0xabcdef")]);

        let result = map_evm_quote_data(&quote_response, None).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.value, BigUint::parse_bytes(b"1000000000000000000", 10).unwrap());
        assert_eq!(result.data, "0xabcdef");
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());
    }

    #[test]
    fn test_map_evm_quote_data_with_approval() {
        let approval = ApprovalData::make("0xtoken", "0xrouter", BigUint::from(1000u64), false);

        let quote_response = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction_with_gas("swap", "0xrouter", "0", "0xabcdef", Some(482935))]);
        let result = map_evm_quote_data(&quote_response, Some(approval.clone())).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.approval, Some(approval.clone()));
        assert_eq!(result.gas_limit, Some("724402".to_string()));

        let quote_response = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("swap", "0xrouter", "0", "0xabcdef")]);
        let result = map_evm_quote_data(&quote_response, Some(approval)).unwrap();

        assert_eq!(result.gas_limit, Some(DEFAULT_EVM_SWAP_GAS_LIMIT.to_string()));
    }

    #[test]
    fn test_map_tron_quote_data() {
        let quote_response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_usdt_to_base_usdc.json")).unwrap();
        let result = map_tron_quote_data(&quote_response, None).unwrap();

        assert_eq!(result.to, "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN");
        assert_eq!(result.value, BigUint::from(0u64));
        assert_eq!(
            result.data,
            "e8017952000000000000000000000000f70da97812cb96acdf810712aa562db8dfa3dbef000000000000000000000000a614f803b6fd780986a42c78ec9c7f77e6ded13c00000000000000000000000000000000000000000000000000000000000f42407ea7c6b23ebd61b4a4b5802cfd9ca2ba44bf096a67858bb0efff82111cf096c0"
        );
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());

        let approval = ApprovalData::make("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t", "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN", BigUint::from(1000000u64), true);
        let result = map_tron_quote_data(&quote_response, Some(approval.clone())).unwrap();
        assert_eq!(result.approval, Some(approval));
        assert_eq!(result.gas_limit, Some(DEFAULT_TRON_SWAP_ENERGY_LIMIT.to_string()));

        let native_quote: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_to_base_usdc.json")).unwrap();
        let native = map_tron_quote_data(&native_quote, None).unwrap();
        assert_eq!(native.to, "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN");
        assert_eq!(native.value, BigUint::from(10000000u64));
        assert!(native.approval.is_none());

        assert_eq!(map_evm_quote_data(&quote_response, None).unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_map_ton_quote_data() {
        let quote_response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_ton_to_base_usdc.json")).unwrap();
        let result = map_ton_quote_data(&quote_response).unwrap();

        assert_eq!(result.to, "EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs");
        assert_eq!(result.value, BigUint::from(5_000_000_000u64));
        assert!(result.data.starts_with("te6cckEBAQEASAAAjAAAAAAweGVk"));
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());

        assert_eq!(map_evm_quote_data(&quote_response, None).unwrap_err(), SwapperError::InvalidRoute);
        let tron_quote: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_to_base_usdc.json")).unwrap();
        assert_eq!(map_ton_quote_data(&tron_quote).unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_map_swap_result() {
        let cross_chain_response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_arb_eth_to_base_eth.json")).unwrap();
        let result = map_swap_result(cross_chain_response.requests.first().unwrap());

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Arbitrum));
        assert_eq!(metadata.from_value, BigUint::from(60000000000000u64));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Base));
        assert_eq!(metadata.to_value, BigUint::from(49426938842266u64));
        assert_eq!(metadata.provider, Some("relay".to_string()));

        let same_chain_response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_base_eth_to_wsteth.json")).unwrap();
        let result = map_swap_result(same_chain_response.requests.first().unwrap());

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Base));
        assert_eq!(metadata.from_value, BigUint::from(1366348234320898u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Base, "0xc1CBa3fCea344f92D9239c08C0568f6F2F0ee452"));
        assert_eq!(metadata.to_value, BigUint::from(1101293561931134u64));

        let ton_response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_ton_to_robinhood.json")).unwrap();
        let result = map_swap_result(ton_response.requests.first().unwrap());

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Ton));
        assert_eq!(metadata.from_value, BigUint::from(2172206291u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Robinhood, "0x3B542B9B72441e4BA0E70885f983075C51ea5c16"));
        assert_eq!(metadata.to_value, BigUint::parse_bytes(b"201884432306130998993971", 10).unwrap());

        let pending = map_swap_result(&RelayRequest::mock_with_status(RelayStatus::Pending));
        assert_eq!(pending.status, SwapStatus::Pending);
        assert!(pending.metadata.is_none());

        let failed = map_swap_result(&RelayRequest::mock_with_status(RelayStatus::Failure));
        assert_eq!(failed.status, SwapStatus::Failed);
        assert!(failed.metadata.is_none());
    }

    #[test]
    fn test_map_quote_data_without_step_data() {
        let quote_response = RelayQuoteResponse::mock_with_steps(vec![Step::mock_empty("approve", "transaction")]);

        assert!(map_evm_quote_data(&quote_response, None).is_err());
    }
}
