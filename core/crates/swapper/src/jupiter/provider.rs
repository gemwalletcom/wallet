use super::{
    PROGRAM_ADDRESS,
    client::JupiterClient,
    model::{BuildRequest, BuildResponse},
    transaction::{MAX_COMPUTE_UNIT_LIMIT, MAX_TRANSACTION_SIZE, buffered_compute_unit_limit},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    error::INVALID_ADDRESS, fees::default_referral_fees,
};
use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use gem_encoding::encode_base64;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcResult};
use gem_solana::{
    SolanaAccountEncoding, SolanaRpc, TOKEN_PROGRAM, USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS, get_pubkey_by_str,
    models::{AccountData, SimulateTransactionResult, ValueResult},
    token_account::get_token_account,
};
use primitives::{AssetId, Chain};
use std::collections::HashSet;

const MAX_ACCOUNTS: u8 = 64;

#[derive(Debug)]
pub struct Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub provider: ProviderType,
    pub fee_mints: HashSet<&'static str>,
    http_client: JupiterClient<C>,
    rpc_client: JsonRpcClient<R>,
}

impl<C, R> Jupiter<C, R>
where
    C: Client + Clone + Send + Sync + 'static,
    R: Client + Clone + Send + Sync + 'static,
{
    pub fn with_clients(http_client: JupiterClient<C>, rpc_client: JsonRpcClient<R>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Jupiter),
            fee_mints: HashSet::from([USDC_TOKEN_MINT, USDT_TOKEN_MINT, USDS_TOKEN_MINT, WSOL_TOKEN_ADDRESS]),
            http_client,
            rpc_client,
        }
    }

    pub fn get_asset_address(&self, asset_id: &str) -> Result<String, SwapperError> {
        get_pubkey_by_str(asset_id)
            .map(|address| address.to_string())
            .ok_or_else(|| SwapperError::ComputeQuoteError(format!("{}: {asset_id}", INVALID_ADDRESS)))
    }

    fn get_fee_mint(&self, input: &str, output: &str) -> String {
        if self.fee_mints.contains(output) {
            return output.to_string();
        }
        input.to_string()
    }

    fn get_fee_token_account(&self, mint: &str, token_program: &str) -> Result<String, SwapperError> {
        let fee = default_referral_fees().solana;
        if fee.address.is_empty() {
            return Ok(String::new());
        }
        get_token_account(&fee.address, mint, token_program).map_err(SwapperError::from)
    }

    async fn fetch_token_program(&self, mint: &str) -> Result<String, SwapperError> {
        let request = SolanaRpc::GetAccountInfo(mint.to_string(), SolanaAccountEncoding::Base64);
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> = self.rpc_client.request_with_cache(&request, Some(u64::MAX)).await.map_err(SwapperError::from)?;
        let value = rpc_result.take()?;

        value
            .value
            .map(|account| account.owner)
            .ok_or_else(|| SwapperError::compute_quote_error("Unable to fetch the fee token program"))
    }

    async fn fetch_fee_account(&self, input_mint: &str, output_mint: &str) -> Result<String, SwapperError> {
        let fee_mint = self.get_fee_mint(input_mint, output_mint);
        if self.fee_mints.contains(fee_mint.as_str()) {
            return self.get_fee_token_account(&fee_mint, TOKEN_PROGRAM);
        }

        let token_program = self.fetch_token_program(&fee_mint).await?;
        let fee_account = self.get_fee_token_account(&fee_mint, &token_program)?;
        if fee_account.is_empty() {
            return Ok(fee_account);
        }

        let request = SolanaRpc::GetAccountInfo(fee_account.clone(), SolanaAccountEncoding::Base64);
        let rpc_result: JsonRpcResult<ValueResult<Option<AccountData>>> = self.rpc_client.request_with_cache(&request, None).await.map_err(SwapperError::from)?;
        let exists = match rpc_result {
            JsonRpcResult::Value(response) => response.result.value.is_some(),
            JsonRpcResult::Error(_) => false,
        };
        if exists { Ok(fee_account) } else { Ok(String::new()) }
    }

    async fn get_build(&self, request: &QuoteRequest, input_mint: &str, output_mint: &str, fee_account: &str) -> Result<BuildResponse, SwapperError> {
        let fee = default_referral_fees().solana;
        if fee.bps > 0 && fee_account.is_empty() {
            return Err(SwapperError::compute_quote_error("Jupiter referral fee account is unavailable"));
        }

        let build = self
            .http_client
            .get_build(BuildRequest {
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                amount: request.value.clone(),
                taker: request.wallet_address.clone(),
                slippage_bps: request.options.slippage.bps,
                platform_fee_bps: fee.bps,
                fee_account: fee_account.to_string(),
                max_accounts: MAX_ACCOUNTS,
            })
            .await?;
        build.validate(
            input_mint,
            output_mint,
            &request.value,
            &request.wallet_address,
            request.options.slippage.bps,
            fee.bps,
            fee_account,
        )?;
        let transaction = build.transaction_bytes(&request.wallet_address, MAX_COMPUTE_UNIT_LIMIT)?;
        if transaction.len() > MAX_TRANSACTION_SIZE {
            return Err(SwapperError::compute_quote_error("Jupiter transaction exceeds Solana's size limit"));
        }
        Ok(build)
    }

    async fn simulate(&self, build: &BuildResponse, wallet_address: &str) -> Result<u32, SwapperError> {
        let transaction = build.transaction_bytes(wallet_address, MAX_COMPUTE_UNIT_LIMIT)?;
        if transaction.len() > MAX_TRANSACTION_SIZE {
            return Err(SwapperError::transaction_error("Jupiter transaction exceeds Solana's size limit"));
        }

        let request = SolanaRpc::SimulateTransaction(encode_base64(&transaction));
        let response: ValueResult<SimulateTransactionResult> = self.rpc_client.request(request).await.map_err(SwapperError::transaction_error)?;
        if let Some(error) = response.value.err {
            return Err(SwapperError::transaction_error(error));
        }
        let units_consumed = response
            .value
            .units_consumed
            .ok_or_else(|| SwapperError::transaction_error("Solana simulation did not return consumed compute units"))?;
        buffered_compute_unit_limit(units_consumed)
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

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let input_mint = self.get_asset_address(&request.from_asset.id)?;
        let output_mint = self.get_asset_address(&request.to_asset.id)?;
        let fee_account = self.fetch_fee_account(&input_mint, &output_mint).await?;
        let build = self.get_build(request, &input_mint, &output_mint, &fee_account).await?;
        let out_amount = build.out_amount.parse::<U256>().map_err(SwapperError::compute_quote_error)?;
        let route_data = serde_json::to_string(&build).map_err(SwapperError::compute_quote_error)?;

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: out_amount.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: AssetId::from(Chain::Solana, Some(input_mint)),
                    output: AssetId::from(Chain::Solana, Some(output_mint)),
                    route_data,
                }],
                slippage_bps: build.slippage_bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let input_mint = route.input.token_id.as_deref().ok_or(SwapperError::InvalidRoute)?;
        let output_mint = route.output.token_id.as_deref().ok_or(SwapperError::InvalidRoute)?;
        let build: BuildResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let fee = default_referral_fees().solana;
        let fee_account = self.fetch_fee_account(input_mint, output_mint).await?;
        build.validate(
            input_mint,
            output_mint,
            &quote.request.value,
            &quote.request.wallet_address,
            quote.request.options.slippage.bps,
            fee.bps,
            &fee_account,
        )?;
        let gas_limit = self.simulate(&build, &quote.request.wallet_address).await?;
        let transaction = build.transaction_bytes(&quote.request.wallet_address, gas_limit)?;
        if transaction.len() > MAX_TRANSACTION_SIZE {
            return Err(SwapperError::transaction_error("Jupiter transaction exceeds Solana's size limit"));
        }

        Ok(SwapperQuoteData::new_contract(
            PROGRAM_ADDRESS.to_string(),
            String::new(),
            encode_base64(&transaction),
            None,
            Some(gas_limit.to_string()),
        ))
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use gem_encoding::decode_base64;
    use primitives::AssetId;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_jupiter_provider_fetch_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Jupiter::new(rpc_provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string()))),
            wallet_address: "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".to_string(),
            destination_address: "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".to_string(),
            value: "1000000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);

        let route = &quote.data.routes[0];
        assert_eq!(route.input, AssetId::from(Chain::Solana, Some(WSOL_TOKEN_ADDRESS.to_string())));
        assert_eq!(route.output, AssetId::from(Chain::Solana, Some(USDC_TOKEN_MINT.to_string())));

        let build: BuildResponse = serde_json::from_str(&route.route_data)?;
        assert_eq!(build.input_mint, WSOL_TOKEN_ADDRESS);
        assert_eq!(build.output_mint, USDC_TOKEN_MINT);

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.to, PROGRAM_ADDRESS);
        assert!(decode_base64(&quote_data.data).unwrap().len() <= MAX_TRANSACTION_SIZE);

        Ok(())
    }
}
