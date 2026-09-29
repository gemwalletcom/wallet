use std::str::FromStr;

use gem_tron::address::TronAddress;
use num_bigint::BigUint;
use primitives::{
    AssetId, SwapProvider, TransactionSwapMetadata,
    swap::{ApprovalData, SwapPartnerTransaction, SwapReferralFee},
};

use super::{
    asset::map_currency_to_asset_id,
    chain::RelayChain,
    model::{RelayAppFees, RelayCurrency, RelayCurrencyDetail, RelayQuoteResponse, RelayRequest},
};
use crate::{
    SwapResult, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, DEFAULT_TRON_SWAP_ENERGY_LIMIT, get_swap_gas_limit_with_approval},
};

pub fn validate_quote_response(from_chain: RelayChain, quote_response: &RelayQuoteResponse, value: &BigUint) -> Result<(), SwapperError> {
    match from_chain {
        RelayChain::Bitcoin => map_bitcoin_quote_data(quote_response, value).map(drop),
        RelayChain::Evm(_) | RelayChain::Tron | RelayChain::Solana | RelayChain::Ton => Ok(()),
    }
}

pub fn map_bitcoin_quote_data(quote_response: &RelayQuoteResponse, value: &BigUint) -> Result<SwapperQuoteData, SwapperError> {
    let (deposit_address, deposit) = quote_response.get_deposit_address().ok_or(SwapperError::InvalidRoute)?;
    if deposit.amount != value.to_string() {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(SwapperQuoteData::new_transfer(deposit_address.to_string(), value.clone(), None))
}

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
        Some(TransactionSwapMetadata {
            from_asset: map_currency_asset_id(&currency_in.currency)?,
            from_value: BigUint::from_str(currency_in.amount.as_deref()?).ok()?,
            to_asset: map_currency_asset_id(&currency_out.currency)?,
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

pub fn map_partner_transaction(request: &RelayRequest) -> Option<SwapPartnerTransaction> {
    let data = request.data.as_ref()?;
    let route = data.route.as_ref()?;
    let currency_in = route.actual.as_ref().and_then(|actual| actual.currency_in()).or_else(|| route.quoted.as_ref()?.currency_in())?;
    let currency_out = route.actual.as_ref().and_then(|actual| actual.currency_out()).or_else(|| route.quoted.as_ref()?.currency_out())?;
    Some(SwapPartnerTransaction {
        provider: SwapProvider::Relay,
        provider_transaction_id: request.id.clone(),
        status: request.status.clone().into_swap_status(),
        from_asset_id: map_currency_asset_id(&currency_in.currency)?,
        from_value: currency_in.amount.clone()?,
        from_amount_usd: map_amount_usd(currency_in),
        to_asset_id: map_currency_asset_id(&currency_out.currency)?,
        to_value: currency_out.amount.clone()?,
        to_amount_usd: map_amount_usd(currency_out),
        referral_fee: data.app_fees.as_ref().and_then(map_referral_fee),
        from_transaction_hash: data.in_txs.first().and_then(|transaction| transaction.tx_hash.clone()),
        to_transaction_hash: data.out_txs.first().and_then(|transaction| transaction.tx_hash.clone()),
    })
}

fn map_referral_fee(app_fees: &RelayAppFees) -> Option<SwapReferralFee> {
    if app_fees.actual.is_empty() {
        return None;
    }
    let value = app_fees.actual.iter().map(|fee| BigUint::from_str(fee.amount.as_deref()?).ok()).sum::<Option<BigUint>>()?;
    let amount_usd = app_fees.actual.iter().map(|fee| fee.amount_usd.as_deref()?.parse::<f64>().ok()).sum::<Option<f64>>();
    Some(SwapReferralFee {
        asset_id: map_currency_asset_id(app_fees.currency.as_ref()?)?,
        value: value.to_string(),
        amount_usd,
    })
}

fn map_currency_asset_id(currency: &RelayCurrency) -> Option<AssetId> {
    let chain = RelayChain::from_chain_id(currency.chain_id)?.to_chain();
    Some(map_currency_to_asset_id(chain, &currency.address))
}

fn map_amount_usd(detail: &RelayCurrencyDetail) -> Option<f64> {
    detail.amount_usd.as_deref()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::{RelayQuoteResponse, RelayRequest, RelayRequestsResponse, RelayStatus, Step};
    use primitives::{AssetId, Chain, asset_constants::BASE_USDC_ASSET_ID, swap::SwapStatus};

    #[test]
    fn test_map_bitcoin_quote_data() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_btc_to_base_usdc.json")).unwrap();
        let value = BigUint::from(2_000_000u64);
        assert_eq!(
            map_bitcoin_quote_data(&response, &value).unwrap(),
            SwapperQuoteData::new_transfer("bc1qa0z550eacdsxytnxf9yk7xvvhqzqvlx2tl44z2".to_string(), value.clone(), None)
        );
        assert_eq!(map_bitcoin_quote_data(&response, &BigUint::from(2_000_001u64)), Err(SwapperError::InvalidRoute));

        let mut missing = response.clone();
        missing.steps[0].deposit_address = None;
        assert_eq!(map_bitcoin_quote_data(&missing, &value), Err(SwapperError::InvalidRoute));
        let evm = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", "0xrouter", "0", "0x")]);
        assert_eq!(map_bitcoin_quote_data(&evm, &value), Err(SwapperError::InvalidRoute));
        assert_eq!(validate_quote_response(RelayChain::Bitcoin, &evm, &value), Err(SwapperError::InvalidRoute));
        assert_eq!(validate_quote_response(RelayChain::Tron, &evm, &value), Ok(()));
        assert_eq!(validate_quote_response(RelayChain::Bitcoin, &response, &value), Ok(()));
    }

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
    fn test_map_partner_transaction() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/partner_requests.json")).unwrap();
        let transactions: Vec<SwapPartnerTransaction> = response.requests.iter().filter_map(map_partner_transaction).collect();

        assert_eq!(
            transactions[0],
            SwapPartnerTransaction {
                provider: SwapProvider::Relay,
                provider_transaction_id: "0x5b1b1f8f5a0a2bd6c2e0e6f8a1f3a3c4c1d9e3f2b6a7c8d9e0f1a2b3c4d5e6f7".to_string(),
                status: SwapStatus::Completed,
                from_asset_id: BASE_USDC_ASSET_ID.clone(),
                from_value: "109077539".to_string(),
                from_amount_usd: Some(109.07),
                to_asset_id: AssetId::from_chain(Chain::Bitcoin),
                to_value: "137291".to_string(),
                to_amount_usd: Some(108.79),
                referral_fee: Some(SwapReferralFee {
                    asset_id: BASE_USDC_ASSET_ID.clone(),
                    value: "545387".to_string(),
                    amount_usd: Some(0.545),
                }),
                from_transaction_hash: Some("0x2d0f7d6e4b5c3a291807f6e5d4c3b2a19087f6e5d4c3b2a19087f6e5d4c3b2a1".to_string()),
                to_transaction_hash: Some("5f3c1b0a9e8d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b".to_string()),
            }
        );

        assert_eq!(transactions[1].status, SwapStatus::Pending);
        assert_eq!(transactions[1].to_asset_id, AssetId::from_token(Chain::Base, "0xc1CBa3fCea344f92D9239c08C0568f6F2F0ee452"));
        assert_eq!(transactions[1].to_value, "1101293561931134");
        assert_eq!(transactions[1].referral_fee, None);

        assert_eq!(transactions[2].status, SwapStatus::Refunded);
        assert_eq!(transactions[2].from_transaction_hash.as_deref(), Some("0x4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b"));
        assert_eq!(transactions[2].to_transaction_hash, None);
    }

    #[test]
    fn test_map_bitcoin_swap_result() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_btc_to_robinhood.json")).unwrap();
        assert_eq!(
            map_swap_result(&response.requests[0]),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Bitcoin),
                    from_value: BigUint::from(75_357u64),
                    to_asset: AssetId::from_chain(Chain::Robinhood),
                    to_value: BigUint::from(22_836_941_417_936_141u64),
                    provider: Some("relay".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_base_usdc_to_btc.json")).unwrap();
        assert_eq!(
            map_swap_result(&response.requests[0]),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: BASE_USDC_ASSET_ID.clone(),
                    from_value: BigUint::from(109_077_539u64),
                    to_asset: AssetId::from_chain(Chain::Bitcoin),
                    to_value: BigUint::from(137_291u64),
                    provider: Some("relay".to_string()),
                }),
                eta_in_seconds: None,
            }
        );
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
