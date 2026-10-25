use async_trait::async_trait;
use futures::try_join;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use gem_solana::{
    JUPITER_PROGRAM_ID, MAX_COMPUTE_UNIT_LIMIT, SolanaAccountEncoding, SolanaRpc, SolanaTokenProgramIdExt, TOKEN_PROGRAM, get_pubkey_by_str,
    models::{AccountData, ValueResult},
    token_account::get_token_account,
};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, SolanaTokenProgramId};

use super::{client::JupiterClient, model::BuildRequest};
use crate::{
    Contracts, FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData,
    error::INVALID_ADDRESS,
    fees::{SOLANA_FEE_MINTS, on_chain_referral_fees},
};

const MAX_ACCOUNTS: u8 = 64;
#[derive(Debug)]
pub struct Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    provider: ProviderType,
    http_client: JupiterClient<C>,
    rpc_client: JsonRpcClient<R>,
}

impl<C, R> Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub(super) fn with_clients(http_client: JupiterClient<C>, rpc_client: JsonRpcClient<R>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Jupiter),
            http_client,
            rpc_client,
        }
    }

    fn asset_mint(asset_id: &str) -> Result<String, SwapperError> {
        get_pubkey_by_str(asset_id)
            .map(|address| address.to_string())
            .ok_or_else(|| SwapperError::compute_quote_error(format!("{INVALID_ADDRESS}: {asset_id}")))
    }

    async fn get_token_program(&self, asset: &SwapperQuoteAsset) -> Result<String, SwapperError> {
        if let Some(program) = SolanaTokenProgramId::from_asset_type(&asset.asset_type) {
            return Ok(program.program_id().to_base58());
        }

        let request = SolanaRpc::GetAccountInfo(Self::asset_mint(&asset.id)?, SolanaAccountEncoding::Base64);
        let value: ValueResult<Option<AccountData>> = self.rpc_client.request(request).await?;

        value.value.map(|account| account.owner).ok_or_else(|| SwapperError::compute_quote_error("Unable to fetch the fee token program"))
    }

    async fn get_referral_account(&self, from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset, referral_address: &str) -> Result<String, SwapperError> {
        let (input_mint, output_mint) = (Self::asset_mint(&from_asset.id)?, Self::asset_mint(&to_asset.id)?);
        if let Some(mint) = [&output_mint, &input_mint].into_iter().find(|mint| SOLANA_FEE_MINTS.contains(&mint.as_str())) {
            return Ok(get_token_account(referral_address, mint, TOKEN_PROGRAM)?);
        }

        let (input_program, output_program) = try_join!(self.get_token_program(from_asset), self.get_token_program(to_asset))?;
        let candidates = vec![get_token_account(referral_address, &input_mint, &input_program)?, get_token_account(referral_address, &output_mint, &output_program)?];
        let accounts: ValueResult<Vec<Option<AccountData>>> = self.rpc_client.request(SolanaRpc::GetMultipleAccounts(candidates.clone())).await?;
        candidates
            .into_iter()
            .zip(accounts.value)
            .find_map(|(account, data)| data.map(|_| account))
            .ok_or_else(|| SwapperError::compute_quote_error("Jupiter referral fee account is unavailable"))
    }
}

#[async_trait]
impl<C, R> Swapper for Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Solana)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    fn contracts(&self, _from_asset: &AssetId) -> Contracts {
        Contracts::new(vec![JUPITER_PROGRAM_ID])
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let input_mint = Self::asset_mint(&request.from_asset.id)?;
        let output_mint = Self::asset_mint(&request.to_asset.id)?;
        let referral_fee = on_chain_referral_fees(request).solana;
        let fee_account = self.get_referral_account(&request.from_asset, &request.to_asset, &referral_fee.address).await?;
        let build_request = BuildRequest {
            input_mint: input_mint.clone(),
            output_mint: output_mint.clone(),
            amount: request.value.to_string(),
            taker: request.wallet_address.clone(),
            slippage_bps: request.options.slippage.bps,
            platform_fee_bps: referral_fee.bps,
            fee_account,
            max_accounts: MAX_ACCOUNTS,
        };
        let build = self.http_client.get_build(&build_request).await?;
        let to_value = build.out_amount.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
        let slippage_bps = build.slippage_bps;
        let route_data = build.into_transaction(&build_request.taker, &build_request.fee_account)?;

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: AssetId::from(Chain::Solana, Some(input_mint)),
                    output: AssetId::from(Chain::Solana, Some(output_mint)),
                    route_data,
                }],
                slippage_bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;

        Ok(SwapperQuoteData::new_contract(
            JUPITER_PROGRAM_ID.to_string(),
            BigUint::ZERO,
            route.route_data.clone(),
            None,
            Some(MAX_COMPUTE_UNIT_LIMIT.to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_dex::testkit::mock_rpc_client;
    use gem_client::testkit::MockClient;
    use gem_solana::{USDC_TOKEN_MINT, token_account::get_token_account};
    use primitives::swap::SOLANA_REFERRAL_ADDRESS;
    use serde_json::{Value, json};
    use std::{
        collections::HashMap,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    const JLP_MINT: &str = "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4";
    const WIF_MINT: &str = "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm";

    fn token(mint: &str) -> SwapperQuoteAsset {
        SwapperQuoteAsset::from(AssetId::from(Chain::Solana, Some(mint.to_string())))
    }

    fn referral_account(mint: &str) -> String {
        get_token_account(SOLANA_REFERRAL_ADDRESS, mint, TOKEN_PROGRAM).unwrap()
    }

    fn jupiter(existing: &[&str], requests: Arc<AtomicUsize>) -> Jupiter<MockClient, MockClient> {
        let accounts: HashMap<String, Value> = existing.iter().map(|mint| (referral_account(mint), json!({ "data": ["", "base64"], "owner": TOKEN_PROGRAM }))).collect();
        Jupiter::with_clients(JupiterClient::new(MockClient::new()), mock_rpc_client(accounts, requests))
    }

    async fn account(jupiter: Jupiter<MockClient, MockClient>, from: &str, to: &str) -> Result<String, SwapperError> {
        jupiter.get_referral_account(&token(from), &token(to), SOLANA_REFERRAL_ADDRESS).await
    }

    #[tokio::test]
    async fn test_referral_account_uses_whichever_side_has_one() {
        let requests = Arc::new(AtomicUsize::new(0));

        assert_eq!(account(jupiter(&[], requests.clone()), JLP_MINT, USDC_TOKEN_MINT).await, Ok(referral_account(USDC_TOKEN_MINT)));
        assert_eq!(requests.load(Ordering::SeqCst), 0);
        assert_eq!(account(jupiter(&[WIF_MINT], requests.clone()), JLP_MINT, WIF_MINT).await, Ok(referral_account(WIF_MINT)));
        assert_eq!(account(jupiter(&[JLP_MINT, WIF_MINT], requests.clone()), JLP_MINT, WIF_MINT).await, Ok(referral_account(JLP_MINT)));
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert!(account(jupiter(&[], requests), JLP_MINT, WIF_MINT).await.is_err());
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use gem_encoding::decode_base64;
    use gem_solana::{MAX_TRANSACTION_SIZE, USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS};
    use primitives::AssetId;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_jupiter_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Jupiter::new(rpc_provider).unwrap();

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string()))),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            value: BigUint::from(1000000000u64),
            options: Options::default(),
        };

        let quote = provider.get_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);

        let route = &quote.data.routes[0];
        assert_eq!(route.input, AssetId::from(Chain::Solana, Some(WSOL_TOKEN_ADDRESS.to_string())));
        assert_eq!(route.output, AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string())));

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        let swapper: &dyn Swapper = &provider;
        swapper.verify_contracts(&quote, &quote_data)?;
        assert_eq!(quote_data.to, JUPITER_PROGRAM_ID);
        assert!(decode_base64(&quote_data.data).unwrap().len() <= MAX_TRANSACTION_SIZE);

        Ok(())
    }
}
