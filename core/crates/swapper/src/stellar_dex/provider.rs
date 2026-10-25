use super::model::{StellarRoute, fee_side};
use crate::{FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData, client_factory::create_stellar_client};
use async_trait::async_trait;
use gem_client::Client;
use gem_stellar::{
    models::{AccountResult, StrictSendPathsQuery},
    rpc::StellarClient,
};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, unix_timestamp};
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
        if asset_id.is_native() {
            return Ok(false);
        }
        match self.client.get_account(address.to_string()).await.map_err(SwapperError::transaction_error)? {
            AccountResult::Found(account) => Ok(!account.has_trustline(asset_id)),
            AccountResult::NotFound => Err(SwapperError::transaction_error("Stellar account not found")),
        }
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
        let fee_side = fee_side(&from_asset, &to_asset).ok_or(SwapperError::NotSupportedAsset)?;
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
        let data = route.swap_data(quote, change_trust, unix_timestamp() + SWAP_DEADLINE_SECONDS);

        Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, serde_json::to_string(&data)?, None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperQuoteAsset, models::Options};
    use gem_client::{ClientError, testkit::MockClient};
    use gem_stellar::models::{StellarSwapData, StellarSwapFee};
    use primitives::{asset_constants::STELLAR_USDC_ASSET_ID, swap::STELLAR_REFERRAL_ADDRESS};
    use std::sync::Mutex;

    const WALLET: &str = "GCVQA5JVHNDJFPODJZU6BM7H5344VK2KGBFXSUUEP7ZH3T5DXGMCR6NL";
    const EURC_TOKEN_ID: &str = "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::EURC";
    const XLM_TO_EURC_PATHS: &str = "/paths/strict-send?source_asset_type=native&source_amount=50000&destination_assets=EURC%3AGDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2";

    fn provider(requests: Arc<Mutex<Vec<String>>>) -> StellarDex<MockClient> {
        StellarDex::with_client(StellarClient::new(MockClient::new().with_get(move |path| {
            requests.lock().unwrap().push(path.to_string());
            match path {
                XLM_TO_EURC_PATHS => Ok(include_bytes!("testdata/xlm_to_eurc_paths.json").to_vec()),
                path if path == format!("/accounts/{WALLET}") => Ok(include_bytes!("testdata/account_with_usdc_trustline.json").to_vec()),
                _ => Err(ClientError::Http { status: 404, body: Vec::new() }),
            }
        })))
    }

    fn request(from_asset: AssetId, to_asset: AssetId, value: u64) -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(from_asset),
            to_asset: SwapperQuoteAsset::from(to_asset),
            wallet_address: WALLET.to_string(),
            destination_address: WALLET.to_string(),
            value: BigUint::from(value),
            options: Options::new_with_slippage(100.into()),
        }
    }

    #[tokio::test]
    async fn test_swap_pays_itself_the_minimum_and_gem_its_fee_with_a_trustline() {
        let requests = Arc::default();
        let provider = provider(Arc::clone(&requests));
        let xlm = AssetId::from_chain(Chain::Stellar);
        let eurc = AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID);

        let quote = provider.get_quote(&request(xlm.clone(), eurc, 502_512_562_814)).await.unwrap();
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let data: StellarSwapData = serde_json::from_str(&quote_data.data).unwrap();

        assert_eq!(quote.from_value, BigUint::from(502_512_562_814u64));
        assert_eq!(quote.to_value, BigUint::from(76_932_168_543u64));
        assert_eq!(
            data,
            StellarSwapData {
                send_amount: 500_000_000_000,
                destination_min: 76_162_846_857,
                path: vec![STELLAR_USDC_ASSET_ID.clone()],
                change_trust: true,
                fee: Some(StellarSwapFee {
                    destination: STELLAR_REFERRAL_ADDRESS.to_string(),
                    asset: xlm,
                    amount: 2_512_562_814,
                }),
                max_time: data.max_time,
            }
        );
        assert!(data.max_time.abs_diff(unix_timestamp() + SWAP_DEADLINE_SECONDS) <= 5);
        assert_eq!(*requests.lock().unwrap(), vec![XLM_TO_EURC_PATHS.to_string(), format!("/accounts/{WALLET}")]);
    }

    #[tokio::test]
    async fn test_quote_data_refuses_a_route_that_spends_another_amount() {
        let provider = provider(Arc::default());
        let quote = provider
            .get_quote(&request(AssetId::from_chain(Chain::Stellar), AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID), 502_512_562_814))
            .await
            .unwrap();

        let tampered = Quote {
            from_value: BigUint::from(502_512_562_815u64),
            ..quote
        };

        assert_eq!(provider.get_quote_data(&tampered, FetchQuoteData::None).await, Err(SwapperError::InvalidRoute));
    }

    #[tokio::test]
    async fn test_pairs_without_a_fee_asset_are_left_out() {
        let requests = Arc::default();
        let provider = provider(Arc::clone(&requests));
        let eurc = AssetId::from_token(Chain::Stellar, EURC_TOKEN_ID);

        assert_eq!(provider.get_quote(&request(eurc, STELLAR_USDC_ASSET_ID.clone(), 10_000_000)).await, Err(SwapperError::NotSupportedAsset));
        assert!(requests.lock().unwrap().is_empty());
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use gem_stellar::models::StellarSwapData;
    use primitives::asset_constants::STELLAR_USDC_ASSET_ID;

    const WALLET: &str = "GCVQA5JVHNDJFPODJZU6BM7H5344VK2KGBFXSUUEP7ZH3T5DXGMCR6NL";

    fn request(from_asset: AssetId, to_asset: AssetId, value: u64) -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(from_asset),
            to_asset: SwapperQuoteAsset::from(to_asset),
            wallet_address: WALLET.to_string(),
            destination_address: WALLET.to_string(),
            value: BigUint::from(value),
            options: Options::new_with_slippage(100.into()),
        }
    }

    #[tokio::test]
    async fn test_stellar_dex_quotes_and_builds_xlm_and_usdc() -> Result<(), SwapperError> {
        let provider = StellarDex::new(Arc::new(NativeProvider::default())).unwrap();
        let xlm = AssetId::from_chain(Chain::Stellar);

        for (from_asset, to_asset, value) in [
            (xlm.clone(), STELLAR_USDC_ASSET_ID.clone(), 58_380_000_000),
            (xlm.clone(), STELLAR_USDC_ASSET_ID.clone(), 2_919_000_000_000),
            (STELLAR_USDC_ASSET_ID.clone(), xlm.clone(), 10_000_000_000),
        ] {
            let quote = provider.get_quote(&request(from_asset, to_asset, value)).await?;
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
