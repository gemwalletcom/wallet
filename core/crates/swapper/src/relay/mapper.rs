use std::str::FromStr;

use gem_ton::models::TonConnectMessage;
use gem_tron::models::TriggerSmartContractData;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::{
    AssetId, TransactionSwapMetadata, TransactionSwapReferralFee,
    swap::{ApprovalData, EVM_REFERRAL_ADDRESS},
};

use super::{
    asset::map_currency_to_asset_id,
    chain::RelayChain,
    contracts,
    model::{EvmStepData, RelayQuoteResponse, RelayRequest, RelayRequestAppFees},
};
use crate::{
    Deployment, SwapResult, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, DEFAULT_TRON_SWAP_ENERGY_LIMIT, get_swap_gas_limit_with_approval},
    contract_check::{expected_native_value, verify_value},
};

pub fn validate_quote_response(from_asset: &AssetId, quote_response: &RelayQuoteResponse, value: &BigUint) -> Result<(), SwapperError> {
    let from_chain = RelayChain::from_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
    match from_chain {
        RelayChain::Bitcoin => map_bitcoin_quote_data(quote_response, value).map(drop),
        RelayChain::Evm(_) | RelayChain::Tron | RelayChain::Ton => {
            validate_targets(from_chain, quote_response)?;
            verify_value(&expected_native_value(from_asset, value), &sent_value(from_chain, quote_response)?)
        }
        RelayChain::Solana => Ok(()),
    }
}

fn sent_value(from_chain: RelayChain, quote_response: &RelayQuoteResponse) -> Result<BigUint, SwapperError> {
    match from_chain {
        RelayChain::Evm(_) => evm_value(quote_response.get_evm_step().ok_or(SwapperError::InvalidRoute)?),
        RelayChain::Tron => Ok(tron_value(tron_transaction(quote_response)?)),
        RelayChain::Ton => {
            let messages = ton_messages(quote_response)?;
            TonConnectMessage::total_amount(&messages).map_err(|_| SwapperError::InvalidRoute)
        }
        RelayChain::Bitcoin | RelayChain::Solana => Err(SwapperError::NotSupportedChain),
    }
}

fn evm_value(evm: &EvmStepData) -> Result<BigUint, SwapperError> {
    BigUint::from_str(&evm.value).map_err(SwapperError::compute_quote_error)
}

fn tron_transaction(quote_response: &RelayQuoteResponse) -> Result<&TriggerSmartContractData, SwapperError> {
    let tron = quote_response.get_tron_step().ok_or(SwapperError::InvalidRoute)?;
    tron.trigger_smart_contract().ok_or(SwapperError::InvalidRoute)
}

fn tron_value(transaction: &TriggerSmartContractData) -> BigUint {
    BigUint::from(transaction.call_value.unwrap_or_default())
}

fn ton_messages(quote_response: &RelayQuoteResponse) -> Result<Vec<TonConnectMessage>, SwapperError> {
    let ton = quote_response.get_ton_step().ok_or(SwapperError::InvalidRoute)?;
    ton.messages
        .iter()
        .map(|message| Ok(TonConnectMessage::new(RelayChain::Ton.contract_address(&message.to)?, &BigUint::from_str(&message.value)?, Some(message.body.clone()))))
        .collect()
}

fn validate_targets(from_chain: RelayChain, quote_response: &RelayQuoteResponse) -> Result<(), SwapperError> {
    let chain = from_chain.to_chain();
    let contracts = contracts::contracts(chain).targets;
    for target in quote_response.targets() {
        if !Deployment::contains(chain, &contracts, &from_chain.contract_address(&target)?) {
            return Err(SwapperError::InvalidRoute);
        }
    }
    Ok(())
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
    Ok(SwapperQuoteData::new_contract(evm.to.clone(), evm_value(evm)?, call_data, approval, gas_limit))
}

pub fn map_tron_quote_data(quote_response: &RelayQuoteResponse, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let transaction = tron_transaction(quote_response)?;
    let contract = RelayChain::Tron.contract_address(&transaction.contract_address)?;
    let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_TRON_SWAP_ENERGY_LIMIT);
    Ok(SwapperQuoteData::new_contract(contract, tron_value(transaction), transaction.data.clone(), approval, gas_limit))
}

pub fn map_ton_quote_data(quote_response: &RelayQuoteResponse) -> Result<SwapperQuoteData, SwapperError> {
    TonConnectMessage::swap_quote_data(&ton_messages(quote_response)?).map_err(|_| SwapperError::InvalidRoute)
}

fn map_referral_fee(app_fees: Option<&RelayRequestAppFees>) -> Option<TransactionSwapReferralFee> {
    let app_fees = app_fees?;
    let currency = app_fees.currency.as_ref()?;
    let value: BigUint = app_fees
        .actual
        .iter()
        .filter(|fee| fee.recipient.as_deref().is_some_and(|recipient| recipient.eq_ignore_ascii_case(EVM_REFERRAL_ADDRESS)))
        .filter_map(|fee| fee.amount.as_ref())
        .sum();
    if value.is_zero() {
        return None;
    }
    let chain = RelayChain::from_chain_id(currency.chain_id)?.to_chain();
    Some(TransactionSwapReferralFee {
        asset_id: map_currency_to_asset_id(chain, &currency.address),
        value,
    })
}

pub fn map_swap_result(request: &RelayRequest) -> SwapResult {
    let metadata = request.data.as_ref().and_then(|data| {
        let actual = data.route.as_ref()?.actual.as_ref()?;
        let currency_in = actual.currency_in()?;
        let currency_out = actual.currency_out()?;
        let from_chain = RelayChain::from_chain_id(currency_in.currency.chain_id)?.to_chain();
        let to_chain = RelayChain::from_chain_id(currency_out.currency.chain_id)?.to_chain();
        Some(
            TransactionSwapMetadata::new(
                map_currency_to_asset_id(from_chain, &currency_in.currency.address),
                BigUint::from_str(currency_in.amount.as_deref()?).ok()?,
                map_currency_to_asset_id(to_chain, &currency_out.currency.address),
                BigUint::from_str(currency_out.amount.as_deref()?).ok()?,
                SwapperProvider::Relay,
            )
            .with_referral_fee(map_referral_fee(data.app_fees.as_ref())),
        )
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
    use primitives::{
        AssetId, Chain,
        asset_constants::{BASE_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID},
        swap::SwapStatus,
    };

    #[test]
    fn test_validate_quote_response_binds_targets_to_shipped_contracts() {
        let celo = include_str!("testdata/quote_celo_native_to_bsc_usdt.json");
        let shipped: RelayQuoteResponse = serde_json::from_str(celo).unwrap();
        assert!(validate_quote_response(&AssetId::from_chain(Chain::Celo), &shipped, &BigUint::from(1_000_000_000_000_000_000u64)).is_ok());

        let tron: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_to_base_usdc.json")).unwrap();
        assert!(
            validate_quote_response(&AssetId::from_chain(Chain::Tron), &tron, &BigUint::from(10_000_000u64)).is_ok(),
            "a hex Tron contract matches the shipped base58 depository"
        );

        let foreign: RelayQuoteResponse = serde_json::from_str(&celo.replace("0xccc88a9d1b4ed6b0eaba998850414b24f1c315be", "0x0000000000000000000000000000000000000001")).unwrap();
        assert_eq!(validate_quote_response(&AssetId::from_chain(Chain::Celo), &foreign, &BigUint::ZERO).unwrap_err(), SwapperError::InvalidRoute);

        let mut ton: serde_json::Value = serde_json::from_str(include_str!("testdata/quote_ton_to_base_usdc.json")).unwrap();
        let shipped_ton: RelayQuoteResponse = serde_json::from_value(ton.clone()).unwrap();
        assert!(validate_quote_response(&AssetId::from_chain(Chain::Ton), &shipped_ton, &BigUint::from(5_000_000_000u64)).is_ok());
        let step = ton["steps"].as_array_mut().unwrap().iter_mut().find(|step| step["items"][0]["data"]["messages"].is_array()).unwrap();
        let messages = step["items"][0]["data"]["messages"].as_array_mut().unwrap();
        let mut drain = messages[0].clone();
        drain["to"] = serde_json::Value::String("EQBvW8Z5huBkMJYdnfAEM5JqTNkuWX3diqYENkWsIL0XggGG".to_string());
        messages.push(drain);
        let drained: RelayQuoteResponse = serde_json::from_value(ton).unwrap();
        assert_eq!(
            validate_quote_response(&AssetId::from_chain(Chain::Ton), &drained, &BigUint::from(5_000_000_000u64)).unwrap_err(),
            SwapperError::InvalidRoute,
            "a second message paying elsewhere is refused"
        );
    }

    #[test]
    fn test_validate_quote_response_binds_the_native_value_sent() {
        let eth = AssetId::from_chain(Chain::Ethereum);
        let usdc = ETHEREUM_USDC_ASSET_ID.clone();
        let amount = BigUint::from(10_000_000_000_000_000u64);
        let native = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", "0x4cD00E387622C35bDDB9b4c962C136462338BC31", "10000000000000000", "0x")]);
        let drain = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", "0x4cD00E387622C35bDDB9b4c962C136462338BC31", "50000000000000000000", "0x")]);
        let token = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", "0x4cD00E387622C35bDDB9b4c962C136462338BC31", "0", "0x")]);
        let token_with_value = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", "0x4cD00E387622C35bDDB9b4c962C136462338BC31", "1", "0x")]);

        assert_eq!(validate_quote_response(&eth, &native, &amount), Ok(()));
        assert_eq!(
            validate_quote_response(&eth, &drain, &amount),
            Err(SwapperError::TransactionError("swap value mismatch: quote=10000000000000000, provider=50000000000000000000".to_string()))
        );
        assert_eq!(validate_quote_response(&usdc, &token, &BigUint::from(5_000_000u64)), Ok(()));
        assert!(validate_quote_response(&usdc, &token_with_value, &BigUint::from(5_000_000u64)).is_err());

        let tron: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_to_base_usdc.json")).unwrap();
        assert!(validate_quote_response(&AssetId::from_chain(Chain::Tron), &tron, &BigUint::from(9_999_999u64)).is_err());

        let celo: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_celo_native_to_bsc_usdt.json")).unwrap();
        assert_eq!(
            validate_quote_response(&AssetId::from_chain(Chain::Celo), &celo, &BigUint::from(1u32)),
            Ok(()),
            "native CELO is pulled as an ERC-20, so no value is sent"
        );

        let ton: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_ton_to_base_usdc.json")).unwrap();
        assert!(validate_quote_response(&AssetId::from_chain(Chain::Ton), &ton, &BigUint::from(4_000_000_000u64)).is_err());
    }

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
        assert_eq!(validate_quote_response(&AssetId::from_chain(Chain::Bitcoin), &evm, &value), Err(SwapperError::InvalidRoute));
        assert_eq!(validate_quote_response(&AssetId::from_chain(Chain::Tron), &evm, &value), Err(SwapperError::InvalidRoute));
        assert_eq!(validate_quote_response(&AssetId::from_chain(Chain::Solana), &evm, &value), Ok(()));
        assert_eq!(validate_quote_response(&AssetId::from_chain(Chain::Bitcoin), &response, &value), Ok(()));
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
        let body = quote_response.get_ton_step().unwrap().messages[0].body.clone();

        assert_eq!(result.to, "EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs");
        assert_eq!(result.value, BigUint::from(5_000_000_000u64));
        assert_eq!(
            serde_json::from_str::<Vec<TonConnectMessage>>(&result.data).unwrap(),
            vec![TonConnectMessage::new("EQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVIPs".to_string(), &BigUint::from(5_000_000_000u64), Some(body))]
        );
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());

        assert_eq!(map_evm_quote_data(&quote_response, None).unwrap_err(), SwapperError::InvalidRoute);
        let tron_quote: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_to_base_usdc.json")).unwrap();
        assert_eq!(map_ton_quote_data(&tron_quote).unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_map_bitcoin_swap_result() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_btc_to_robinhood.json")).unwrap();
        assert_eq!(
            map_swap_result(&response.requests[0]),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Bitcoin),
                    BigUint::from(75_357u64),
                    AssetId::from_chain(Chain::Robinhood),
                    BigUint::from(22_836_941_417_936_141u64),
                    SwapperProvider::Relay
                )),
                eta_in_seconds: None,
            }
        );
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_base_usdc_to_btc.json")).unwrap();
        assert_eq!(
            map_swap_result(&response.requests[0]),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    BASE_USDC_ASSET_ID.clone(),
                    BigUint::from(109_077_539u64),
                    AssetId::from_chain(Chain::Bitcoin),
                    BigUint::from(137_291u64),
                    SwapperProvider::Relay
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn test_map_swap_result_referral_fee() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_eth_usdc_referral_fee.json")).unwrap();
        let metadata = map_swap_result(response.requests.first().unwrap()).metadata.unwrap();

        assert_eq!(metadata.from_asset, ETHEREUM_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, BigUint::from(350000000u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: ETHEREUM_USDC_ASSET_ID.clone(),
                value: BigUint::from(1750000u64),
            })
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
        assert_eq!(metadata.provider, Some(SwapperProvider::Relay.as_ref().to_string()));

        let without_app_fee_recipient: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_arb_usdc_without_app_fee_recipient.json")).unwrap();
        let result = map_swap_result(without_app_fee_recipient.requests.first().unwrap());

        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.metadata.and_then(|metadata| metadata.referral_fee), None);

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
