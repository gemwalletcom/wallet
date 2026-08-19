use primitives::{TransactionSwapMetadata, swap::ApprovalData};

use super::{
    asset::map_currency_to_asset_id,
    chain::RelayChain,
    model::{RelayQuoteResponse, RelayRequest, StepData},
};
use crate::{
    SwapResult, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, get_swap_gas_limit_with_approval},
};

pub fn map_quote_data(quote_response: &RelayQuoteResponse, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let step_data = quote_response.step_data().ok_or(SwapperError::InvalidRoute)?;

    match step_data {
        StepData::Evm(evm) => {
            let gas_limit = get_swap_gas_limit_with_approval(&approval, evm.gas_limit_with_buffer(), DEFAULT_EVM_SWAP_GAS_LIMIT);
            let call_data = evm.data.clone().unwrap_or_default();
            Ok(SwapperQuoteData::new_contract(evm.to.clone(), evm.value.clone(), call_data, approval, gas_limit))
        }
    }
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
            from_value: currency_in.amount.clone()?,
            to_asset: map_currency_to_asset_id(to_chain, &currency_out.currency.address),
            to_value: currency_out.amount.clone()?,
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

        let result = map_quote_data(&quote_response, None).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.value, "1000000000000000000");
        assert_eq!(result.data, "0xabcdef");
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());
    }

    #[test]
    fn test_map_evm_quote_data_with_approval() {
        let approval = ApprovalData::make("0xtoken", "0xrouter", "1000", false);

        let quote_response = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction_with_gas("swap", "0xrouter", "0", "0xabcdef", Some(482935))]);
        let result = map_quote_data(&quote_response, Some(approval.clone())).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.approval, Some(approval.clone()));
        assert_eq!(result.gas_limit, Some("724402".to_string()));

        let quote_response = RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("swap", "0xrouter", "0", "0xabcdef")]);
        let result = map_quote_data(&quote_response, Some(approval)).unwrap();

        assert_eq!(result.gas_limit, Some(DEFAULT_EVM_SWAP_GAS_LIMIT.to_string()));
    }

    #[test]
    fn test_map_swap_result() {
        let cross_chain_response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_arb_eth_to_base_eth.json")).unwrap();
        let result = map_swap_result(cross_chain_response.requests.first().unwrap());

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Arbitrum));
        assert_eq!(metadata.from_value, "60000000000000");
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Base));
        assert_eq!(metadata.to_value, "49426938842266");
        assert_eq!(metadata.provider, Some("relay".to_string()));

        let same_chain_response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_base_eth_to_wsteth.json")).unwrap();
        let result = map_swap_result(same_chain_response.requests.first().unwrap());

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Base));
        assert_eq!(metadata.from_value, "1366348234320898");
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Base, "0xc1CBa3fCea344f92D9239c08C0568f6F2F0ee452"));
        assert_eq!(metadata.to_value, "1101293561931134");

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

        assert!(map_quote_data(&quote_response, None).is_err());
    }
}
