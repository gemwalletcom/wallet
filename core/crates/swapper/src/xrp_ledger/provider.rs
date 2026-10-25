use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use gem_client::Client;
use gem_xrp::{
    models::{PathFindAmount, PathFindRequest, XrpSwap},
    rpc::XrpClient,
};
use num_bigint::BigUint;
use primitives::{
    Chain,
    asset_constants::{XRP_RLUSD_ASSET_ID, XRP_USDC_ASSET_ID},
    swap::XRP_REFERRAL_FEE_ASSET_IDS,
};

use super::model::{ledger_value, map_swap, path_destination, send_max};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData, client_factory::create_xrp_client, fees::fee_side,
    models::nonzero_swap_value,
};

#[derive(Debug)]
pub struct XrpLedger<C: Client + Clone> {
    provider: ProviderType,
    client: XrpClient<C>,
}

impl XrpLedger<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        Some(Self::with_client(create_xrp_client(rpc_provider).ok()?))
    }
}

impl<C: Client + Clone> XrpLedger<C> {
    pub fn with_client(client: XrpClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::XrpLedger),
            client,
        }
    }
}

#[async_trait]
impl<C> Swapper for XrpLedger<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::assets(Chain::Xrp, [XRP_RLUSD_ASSET_ID.clone(), XRP_USDC_ASSET_ID.clone()])]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let fee_side = fee_side(&*XRP_REFERRAL_FEE_ASSET_IDS, &request.from_asset.asset_id(), &request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let referral_fee = request.on_chain_referral_fee();
        let request = QuoteRequest {
            value: ledger_value(&request.from_asset, nonzero_swap_value(&request.value)?),
            ..request.clone()
        };
        let send_max = send_max(&request, fee_side, &referral_fee);
        let path_request = PathFindRequest::new(
            &request.wallet_address,
            &path_destination(&request),
            PathFindAmount::new(&request.from_asset, Some(&send_max)).map_err(SwapperError::compute_quote_error)?,
            PathFindAmount::new(&request.to_asset, None).map_err(SwapperError::compute_quote_error)?,
        );
        let alternatives = self.client.get_path_alternatives(path_request).await.map_err(SwapperError::compute_quote_error)?;
        let (to_value, swap) = map_swap(&request, fee_side, &referral_fee, send_max, &alternatives)?;

        Quote::from_route(&request, self.provider(), to_value, request.options.slippage.bps, &swap, None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let swap: XrpSwap = quote.route_data()?;
        if swap.input_value(&quote.request.from_asset.asset_id()) != quote.from_value {
            return Err(SwapperError::InvalidRoute);
        }
        let trust_line = match quote.request.to_asset.asset_id().token_id {
            Some(token_id) => !self.client.has_trust_line(&quote.request.wallet_address, &token_id).await.map_err(SwapperError::transaction_error)?,
            None => false,
        };
        let data = serde_json::to_string(&XrpSwap { trust_line, ..swap })?;
        Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, data, None, None))
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{NativeProvider, xrp_ledger::testkit::mock_request};
    use primitives::Asset;

    #[tokio::test]
    async fn test_xrp_ledger_quotes_and_quote_data() -> Result<(), SwapperError> {
        let provider = XrpLedger::new(Arc::new(NativeProvider::default())).unwrap();
        let (xrp, rlusd, usdc) = (Asset::from_chain(Chain::Xrp), Asset::mock_xrp_rlusd(), Asset::mock_xrp_usdc());
        for (from, to, value) in [
            (&xrp, &rlusd, 764_000_000),
            (&xrp, &rlusd, 38_200_000_000),
            (&xrp, &usdc, 764_000_000),
            (&xrp, &usdc, 38_200_000_000),
            (&rlusd, &xrp, 1_000_000_000_000_000_000),
            (&usdc, &xrp, 1_000_000_000_000_000_000),
            (&rlusd, &usdc, 1_000_000_000_000_000_000),
        ] {
            let pair = format!("{} to {}", from.symbol, to.symbol);
            let quote = provider.get_quote(&mock_request(from, to, value)).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            let swap: XrpSwap = serde_json::from_str(&data.data)?;
            println!(
                "{pair}: from_value={} to_value={} send_max={} deliver_min={} fee={:?} paths={}",
                quote.from_value,
                quote.to_value,
                swap.send_max,
                swap.deliver_min,
                swap.fee,
                swap.paths.len()
            );

            assert!(quote.to_value > BigUint::ZERO);
            assert!(swap.deliver_min < swap.amount);
            assert!(!swap.trust_line);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use gem_xrp::models::XrpPathStep;
    use primitives::{
        Asset, AssetId, TransactionSwapReferralFee,
        asset_constants::{XRP_RLUSD_TOKEN_ID, XRP_USDC_TOKEN_ID},
        swap::SwapQuoteDataType,
    };
    use serde_json::json;

    use super::*;
    use crate::xrp_ledger::testkit::{TEST_WALLET, mock_request, mock_xrp_client};

    #[tokio::test]
    async fn test_get_quote() {
        let (xrp, rlusd, usdc) = (Asset::from_chain(Chain::Xrp), Asset::mock_xrp_rlusd(), Asset::mock_xrp_usdc());
        let path = |currency: &str, issuer: Option<&str>| {
            vec![vec![XrpPathStep {
                account: None,
                currency: Some(currency.to_string()),
                issuer: issuer.map(str::to_string),
            }]]
        };
        let cases = [
            (
                mock_request(&xrp, &rlusd, 1_000_000_000),
                include_str!("testdata/xrp_to_rlusd_path_find.json"),
                json!({ "source_account": TEST_WALLET, "destination_account": XRP_RLUSD_TOKEN_ID, "destination_amount": { "currency": "524C555344000000000000000000000000000000", "issuer": XRP_RLUSD_TOKEN_ID, "value": "-1" }, "send_max": "995000000", "ledger_index": "current" }),
                1_306_046_206_636_212_000u64,
                XrpSwap {
                    send_max: BigUint::from(995_000_000u64),
                    amount: BigUint::from(1_306_046_206_636_212_000u64),
                    deliver_min: BigUint::from(1_292_985_744_569_849_000u64),
                    paths: path("524C555344000000000000000000000000000000", Some(XRP_RLUSD_TOKEN_ID)),
                    fee: Some(TransactionSwapReferralFee {
                        asset_id: AssetId::from_chain(Chain::Xrp),
                        value: BigUint::from(5_000_000u64),
                    }),
                    trust_line: false,
                },
            ),
            (
                mock_request(&usdc, &xrp, 1_000_000_000_000_000_000),
                include_str!("testdata/usdc_to_xrp_path_find.json"),
                json!({ "source_account": TEST_WALLET, "destination_account": TEST_WALLET, "destination_amount": "-1", "send_max": { "currency": "5553444300000000000000000000000000000000", "issuer": XRP_USDC_TOKEN_ID, "value": "1000" }, "ledger_index": "current" }),
                756_616_100,
                XrpSwap {
                    send_max: BigUint::from(1_000_000_000_000_000_000u64),
                    amount: BigUint::from(760_418_190u64),
                    deliver_min: BigUint::from(752_852_029u64),
                    paths: path("XRP", None),
                    fee: Some(TransactionSwapReferralFee {
                        asset_id: AssetId::from_chain(Chain::Xrp),
                        value: BigUint::from(3_802_090u64),
                    }),
                    trust_line: false,
                },
            ),
        ];

        for (request, path_find, path_request, to_value, swap) in cases {
            let path_requests = Arc::new(Mutex::new(Vec::new()));
            let provider = XrpLedger::with_client(mock_xrp_client(path_find, true, path_requests.clone()));

            let quote = provider.get_quote(&request).await.unwrap();

            assert_eq!(*path_requests.lock().unwrap(), vec![path_request]);
            assert_eq!(quote.from_value, request.value);
            assert_eq!(quote.to_value, BigUint::from(to_value));
            assert_eq!(quote.route_data::<XrpSwap>().unwrap(), swap);
        }

        let path_requests = Arc::new(Mutex::new(Vec::new()));
        let provider = XrpLedger::with_client(mock_xrp_client(include_str!("testdata/usdc_to_xrp_path_find.json"), true, path_requests.clone()));
        assert_eq!(provider.get_quote(&mock_request(&usdc, &rlusd, 1_000_000_000_000_000_000)).await, Err(SwapperError::NotSupportedAsset));
        assert_eq!(*path_requests.lock().unwrap(), Vec::<serde_json::Value>::new());
    }

    #[tokio::test]
    async fn test_get_quote_data() {
        let (xrp, rlusd, usdc) = (Asset::from_chain(Chain::Xrp), Asset::mock_xrp_rlusd(), Asset::mock_xrp_usdc());
        for (request, path_find, has_rlusd_line, trust_line) in [
            (mock_request(&xrp, &rlusd, 1_000_000_000), include_str!("testdata/xrp_to_rlusd_path_find.json"), false, true),
            (mock_request(&xrp, &rlusd, 1_000_000_000), include_str!("testdata/xrp_to_rlusd_path_find.json"), true, false),
            (mock_request(&usdc, &xrp, 1_000_000_000_000_000_000), include_str!("testdata/usdc_to_xrp_path_find.json"), false, false),
        ] {
            let provider = XrpLedger::with_client(mock_xrp_client(path_find, has_rlusd_line, Arc::default()));
            let quote = provider.get_quote(&request).await.unwrap();

            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
            let tampered = Quote {
                from_value: &quote.from_value + 1u32,
                ..quote.clone()
            };

            assert_eq!((data.data_type.clone(), data.to.as_str(), data.value.clone(), data.gas_limit.clone()), (SwapQuoteDataType::Contract, "", BigUint::ZERO, None));
            assert_eq!(serde_json::from_str::<XrpSwap>(&data.data).unwrap(), XrpSwap { trust_line, ..quote.route_data().unwrap() });
            assert_eq!(provider.get_quote_data(&tampered, FetchQuoteData::None).await, Err(SwapperError::InvalidRoute));
        }
    }
}
