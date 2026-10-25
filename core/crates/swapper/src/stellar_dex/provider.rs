use super::model::StellarRoute;
use crate::{FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData, client_factory::create_stellar_client, fees::fee_side};
use async_trait::async_trait;
use chain_traits::ChainBalances;
use gem_client::Client;
use gem_stellar::{models::StrictSendPathsQuery, rpc::StellarClient};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, swap::STELLAR_REFERRAL_FEE_ASSET_IDS, unix_timestamp};
use std::sync::Arc;

const SWAP_DEADLINE_SECONDS: u64 = 15 * 60;

#[derive(Debug)]
pub struct StellarDex<C: Client> {
    provider: ProviderType,
    client: StellarClient<C>,
}

impl StellarDex<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        Some(Self::with_client(create_stellar_client(rpc_provider).ok()?))
    }
}

impl<C: Client> StellarDex<C> {
    fn with_client(client: StellarClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::StellarDex),
            client,
        }
    }

    async fn needs_trustline(&self, address: &str, asset_id: &AssetId) -> Result<bool, SwapperError> {
        let Some(token_id) = asset_id.token_id.clone() else {
            return Ok(false);
        };
        let balances = self.client.get_balance_tokens(address.to_string(), vec![token_id]).await.map_err(SwapperError::transaction_error)?;
        Ok(!balances.first().is_some_and(|balance| balance.is_active))
    }
}

#[async_trait]
impl<C: Client> Swapper for StellarDex<C> {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Stellar)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let (from_asset, to_asset) = (request.from_asset.asset_id(), request.to_asset.asset_id());
        let fee_side = fee_side(&*STELLAR_REFERRAL_FEE_ASSET_IDS, &from_asset, &to_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let value = request.value_u64()?;
        let referral_fee = request.on_chain_referral_fee();
        let query = StrictSendPathsQuery::new(&from_asset, referral_fee.swap_value(fee_side, value), &to_asset).map_err(SwapperError::compute_quote_error)?;
        let paths = self.client.get_strict_send_paths(query).await.map_err(SwapperError::compute_quote_error)?;
        let route = StellarRoute::best(&paths, value, &referral_fee, fee_side)?;

        Quote::from_route(request, self.provider(), BigUint::from(route.received_value()), request.options.slippage.bps, &route, None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route: StellarRoute = quote.route_data()?;
        if BigUint::from(route.fee.input_value(route.swap_value)) != quote.from_value {
            return Err(SwapperError::InvalidRoute);
        }
        let change_trust = self.needs_trustline(&quote.request.wallet_address, &quote.request.to_asset.asset_id()).await?;
        let data = route.swap_data(&quote.request, change_trust, unix_timestamp() + SWAP_DEADLINE_SECONDS);

        Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, serde_json::to_string(&data)?, None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stellar_dex::testkit::{TEST_WALLET, eurc, mock_request};
    use gem_client::{ClientError, testkit::MockClient};
    use gem_stellar::models::StellarSwapData;
    use primitives::{TransactionSwapReferralFee, asset_constants::STELLAR_USDC_ASSET_ID};
    use std::sync::Mutex;

    const XLM_TO_EURC_PATHS: &str = "/paths/strict-send?source_asset_type=native&source_amount=50000&destination_assets=EURC%3AGDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2";

    fn provider(requests: Arc<Mutex<Vec<String>>>) -> StellarDex<MockClient> {
        StellarDex::with_client(StellarClient::new(MockClient::new().with_get(move |path| {
            requests.lock().unwrap().push(path.to_string());
            match path {
                XLM_TO_EURC_PATHS => Ok(include_bytes!("testdata/xlm_to_eurc_paths.json").to_vec()),
                path if path == format!("/accounts/{TEST_WALLET}") => Ok(include_bytes!("testdata/account_with_usdc_trustline.json").to_vec()),
                _ => Err(ClientError::Http { status: 404, body: Vec::new() }),
            }
        })))
    }

    #[tokio::test]
    async fn test_swap_takes_the_best_path_pays_itself_the_minimum_and_gem_its_fee() {
        let requests = Arc::default();
        let provider = provider(Arc::clone(&requests));
        let xlm = AssetId::from_chain(Chain::Stellar);

        let quote = provider.get_quote(&mock_request(xlm.clone(), eurc(), 502_512_562_814)).await.unwrap();
        let data: StellarSwapData = serde_json::from_str(&provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap().data).unwrap();

        assert_eq!(quote.to_value, BigUint::from(76_932_168_543u64), "the path through USDC receives more than the direct one");
        assert_eq!(
            data,
            StellarSwapData {
                send_amount: 500_000_000_000,
                destination_min: 76_162_846_857,
                path: vec![STELLAR_USDC_ASSET_ID.clone()],
                change_trust: true,
                fee: Some(TransactionSwapReferralFee {
                    asset_id: xlm,
                    value: BigUint::from(2_512_562_814u64),
                }),
                max_time: data.max_time,
            }
        );
        assert!(data.max_time.abs_diff(unix_timestamp() + SWAP_DEADLINE_SECONDS) <= 5);
        assert_eq!(*requests.lock().unwrap(), vec![XLM_TO_EURC_PATHS.to_string(), format!("/accounts/{TEST_WALLET}")]);

        let tampered = Quote {
            from_value: BigUint::from(502_512_562_815u64),
            ..quote
        };
        assert_eq!(provider.get_quote_data(&tampered, FetchQuoteData::None).await, Err(SwapperError::InvalidRoute));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{alien::reqwest_provider::NativeProvider, stellar_dex::testkit::mock_request};
    use gem_stellar::models::StellarSwapData;
    use primitives::asset_constants::STELLAR_USDC_ASSET_ID;

    #[tokio::test]
    async fn test_stellar_dex_quotes_and_builds_xlm_and_usdc() -> Result<(), SwapperError> {
        let provider = StellarDex::new(Arc::new(NativeProvider::default())).unwrap();
        let xlm = AssetId::from_chain(Chain::Stellar);

        for (from_asset, to_asset, value) in [
            (xlm.clone(), STELLAR_USDC_ASSET_ID.clone(), 58_380_000_000),
            (xlm.clone(), STELLAR_USDC_ASSET_ID.clone(), 2_919_000_000_000),
            (STELLAR_USDC_ASSET_ID.clone(), xlm.clone(), 10_000_000_000),
        ] {
            let quote = provider.get_quote(&mock_request(from_asset, to_asset, value)).await?;
            let data: StellarSwapData = serde_json::from_str(&provider.get_quote_data(&quote, FetchQuoteData::None).await?.data)?;
            println!(
                "{} -> {}: {} for {}, path {:?}, minimum {}",
                quote.request.from_asset.id, quote.request.to_asset.id, quote.to_value, quote.from_value, data.path, data.destination_min
            );

            assert!(quote.to_value > BigUint::ZERO);
            assert!(!data.change_trust);
            assert!(data.fee.is_some());
        }
        Ok(())
    }
}
