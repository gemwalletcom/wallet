use super::{
    AppFee, DepositData, DepositMode, NearIntentsClient, NearIntentsExplorer, QuoteRequest as NearQuoteRequest, QuoteResponse, QuoteResponseError, QuoteResponseResult, SwapType, auto_quote_time_chains,
    chain::{self, hypercore, sui},
    deposit_memo_chains, get_asset_id_from_near_asset, get_near_asset_id,
    model::{DEFAULT_WAIT_TIME_MS, DEPOSIT_TYPE_ORIGIN, ExplorerTransaction, RECIPIENT_TYPE_DESTINATION},
    supported_assets,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, SwapResult, SwapResultRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset,
    SwapperQuoteData, amount_to_value,
    client_factory::create_sui_client,
    cross_chain::VaultAddresses,
    fees::DEFAULT_REFERRER,
    fees::default_referral_fees,
    near_intents::client::{base_url, explorer_url},
};
use async_trait::async_trait;
#[cfg(test)]
use chrono::DateTime;
use chrono::{Duration, Utc};
use gem_sui::SuiClient;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::Zero;
use primitives::{
    AssetId, Chain, OptionStringExt, TransactionSwapMetadata, TransactionSwapReferralFee,
    swap::{HUNDRED_PERCENT_IN_BPS, NEAR_REFERRAL_ADDRESS, SwapStatus},
};
use std::str::FromStr;
use std::{fmt::Debug, sync::Arc};

const DEFAULT_DEADLINE_MINUTES: i64 = 30;
const BITCOIN_DEADLINE_MINUTES: i64 = 120;

const TREASURY_ADDRESSES: [&str; 17] = [
    "0x2CfF890f0378a11913B6129B2E97417a2c302680",
    "0x233c5370CCfb3cD7409d9A3fb98ab94dE94Cb4Cd",
    "1C6XJtNXiuXvk4oUAVMkKF57CRpaTrN5Ra",
    "1LxByjYMdnogW9Nc73srT4NCbS8oPVaXvZ",
    "DRmCnxzL9U11EJzLmWkm2ikaZikPFbLuQD",
    "XxA9DbXaFpF4GFY8KUNX7eAxhZPsWtcKhc",
    "LQjEMkuiA2pCwFeUPwsu6ktzUubBVLsahX",
    "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML",
    "intents.near",
    "HWjmoUNYckccg9Qrwi43JTzBcGcM1nbdAtATf9GXmz16",
    "UQAfoBd_f0pIvNpUPAkOguUrFWpGWV9TWBeZs_5TXE95_trZ",
    "GDJ4JZXZELZD737NVFORH4PSSQDWFDZTKW3AIDKHYQG23ZXBPDGGQBJK",
    "0x00ea18889868519abd2f238966cab9875750bb2859ed3a34debec37781520138",
    "0xd1a1c1804e91ba85a569c7f018bb7502d2f13d4742d2611953c9c14681af6446",
    "TX5XiRXdyz7sdFwF5mnhT1QoGCpbkncpke",
    "r9R8jciZBYGq32DxxQrBPi5ysZm67iQitH",
    "addr1v8wfpcg4qfhmnzprzysj6j9c53u5j56j8rvhyjp08s53s6g07rfjm",
];

pub struct NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: NearIntentsClient<C>,
    explorer: NearIntentsExplorer<C>,
    supported_assets: Vec<SwapperChainAsset>,
    sui_client: SuiClient,
}

impl<C> std::fmt::Debug for NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NearIntents")
            .field("provider", &self.provider)
            .field("client", &self.client)
            .field("explorer", &self.explorer)
            .field("supported_assets", &self.supported_assets)
            .field("sui_client", &"SuiClient")
            .finish()
    }
}

impl NearIntents<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        let client = NearIntentsClient::new(RpcClient::new(base_url(), rpc_provider.clone()), None);
        let explorer = NearIntentsExplorer::new(RpcClient::new(explorer_url(), rpc_provider.clone()));
        let sui_client = create_sui_client(rpc_provider.clone()).ok()?;
        Some(Self::with_client(client, explorer, sui_client))
    }
}

impl<C> NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_client(client: NearIntentsClient<C>, explorer: NearIntentsExplorer<C>, sui_client: SuiClient) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::NearIntents),
            client,
            explorer,
            supported_assets: supported_assets(),
            sui_client,
        }
    }
    fn build_app_fee() -> Option<Vec<AppFee>> {
        let fee = default_referral_fees().near;
        if fee.address.is_empty() || fee.bps == 0 {
            return None;
        }
        Some(vec![AppFee { recipient: fee.address, fee: fee.bps }])
    }

    fn build_quote_request(request: &QuoteRequest, dry: bool) -> Result<NearQuoteRequest, SwapperError> {
        let origin_asset = get_near_asset_id(&request.from_asset)?;
        let destination_asset = get_near_asset_id(&request.to_asset)?;
        let deposit_mode = Self::deposit_mode(&request.from_asset);
        let from_chain = request.from_asset.asset_id().chain;
        let to_chain = request.to_asset.asset_id().chain;
        if !chain::supports_destination(to_chain) {
            return Err(SwapperError::NotSupportedAsset);
        }
        let quote_waiting_time_ms = Some(Self::quote_waiting_time(from_chain, to_chain));

        let deadline_minutes = Self::get_deadline_by_chain(from_chain).max(Self::get_deadline_by_chain(to_chain));
        let deadline = (Utc::now() + Duration::minutes(deadline_minutes)).to_rfc3339();
        Ok(NearQuoteRequest {
            origin_asset,
            destination_asset,
            amount: request.value.to_string(),
            referral: DEFAULT_REFERRER.to_string(),
            recipient: request.destination_address.clone(),
            swap_type: chain::swap_type(from_chain),
            slippage_tolerance: request.options.slippage.bps,
            app_fees: Self::build_app_fee(),
            deposit_type: DEPOSIT_TYPE_ORIGIN.to_string(),
            refund_to: request.wallet_address.clone(),
            refund_type: DEPOSIT_TYPE_ORIGIN.to_string(),
            recipient_type: RECIPIENT_TYPE_DESTINATION.to_string(),
            deadline,
            quote_waiting_time_ms,
            dry,
            deposit_mode,
        })
    }

    fn map_transaction_status(status: &str) -> SwapStatus {
        match status {
            "SWAP_COMPLETED" | "SWAP_COMPLETED_TX" | "SUCCESS" => SwapStatus::Completed,
            "REFUNDED" | "SWAP_REFUNDED" => SwapStatus::Refunded,
            "SWAP_FAILED" | "FAILED" | "SWAP_LIQUIDITY_TIMEOUT" | "SWAP_RISK_FAILED" => SwapStatus::Failed,
            "KNOWN_DEPOSIT_TX" | "PENDING_DEPOSIT" | "INCOMPLETE_DEPOSIT" | "PROCESSING" => SwapStatus::Pending,
            _ => SwapStatus::Pending,
        }
    }

    fn build_swap_metadata(transaction: &ExplorerTransaction, status: &SwapStatus) -> Option<TransactionSwapMetadata> {
        let from_asset = get_asset_id_from_near_asset(&transaction.origin_asset)?;
        let from_value = BigUint::from_str(&transaction.amount_in).ok().filter(|value| !value.is_zero())?;
        let to_asset = get_asset_id_from_near_asset(&transaction.destination_asset)?;
        let to_value = BigUint::from_str(&transaction.amount_out).ok().filter(|value| !value.is_zero())?;
        let referral_fee = Self::referral_fee(transaction, &from_asset, &from_value).filter(|_| status.charges_referral_fee());
        Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, SwapperProvider::NearIntents).with_referral_fee(referral_fee))
    }

    fn referral_fee(transaction: &ExplorerTransaction, from_asset: &AssetId, from_value: &BigUint) -> Option<TransactionSwapReferralFee> {
        let app_fee = transaction.app_fees.iter().find(|app_fee| app_fee.recipient.eq_ignore_ascii_case(NEAR_REFERRAL_ADDRESS) && app_fee.fee > 0)?;
        Some(TransactionSwapReferralFee {
            asset_id: from_asset.clone(),
            value: from_value * app_fee.fee / HUNDRED_PERCENT_IN_BPS,
        })
    }

    fn deposit_mode(asset: &SwapperQuoteAsset) -> DepositMode {
        if deposit_memo_chains().contains(&asset.asset_id().chain) { DepositMode::Memo } else { DepositMode::Simple }
    }

    fn quote_waiting_time(from_chain: Chain, to_chain: Chain) -> u32 {
        if auto_quote_time_chains().contains(&from_chain) || auto_quote_time_chains().contains(&to_chain) {
            0
        } else {
            DEFAULT_WAIT_TIME_MS
        }
    }

    fn get_deadline_by_chain(chain: Chain) -> i64 {
        if chain == Chain::Bitcoin { BITCOIN_DEADLINE_MINUTES } else { DEFAULT_DEADLINE_MINUTES }
    }

    async fn build_deposit_data(&self, deposit_memo: Option<String>, from_asset: &SwapperQuoteAsset, wallet_address: &str, deposit_address: &str, amount_in: &str) -> Result<DepositData, SwapperError> {
        match from_asset.chain() {
            Chain::HyperCore => hypercore::build_deposit_data(from_asset, deposit_address, amount_in),
            Chain::Sui => sui::build_deposit_data(&self.sui_client, from_asset, wallet_address, deposit_address, amount_in).await,
            _ => Ok(DepositData {
                to: deposit_address.to_string(),
                value: amount_in.to_string(),
                data: String::new(),
                memo: deposit_memo,
            }),
        }
    }

    fn extract_quote(response: QuoteResponseResult, from_decimals: u32) -> Result<QuoteResponse, SwapperError> {
        let quote_response = match response {
            QuoteResponseResult::Ok(response) => *response,
            QuoteResponseResult::Err(error) => return Err(map_quote_error(&error, from_decimals)),
        };
        validate_minimum_amount(&quote_response.quote.amount_in, &quote_response.quote.amount_out, &quote_response.quote.min_amount_out, &quote_response.quote.withdraw_fee)?;
        Ok(quote_response)
    }
}

fn validate_minimum_amount(amount_in: &BigUint, amount_out: &BigUint, min_amount_out: &BigUint, withdrawal_fee: &BigUint) -> Result<(), SwapperError> {
    if min_amount_out > amount_out {
        return Err(SwapperError::ComputeQuoteError("Near Intents returned an invalid minimum output".into()));
    }

    let withdrawal_budget = amount_out - min_amount_out;
    if withdrawal_fee <= &withdrawal_budget {
        return Ok(());
    }
    if withdrawal_budget.is_zero() {
        return Err(SwapperError::NoQuoteAvailable);
    }

    let minimum_amount = (amount_in * withdrawal_fee).div_ceil(&withdrawal_budget);
    Err(SwapperError::InputAmountError {
        min_amount: Some(minimum_amount.to_string()),
    })
}

fn map_quote_error(error: &QuoteResponseError, from_decimals: u32) -> SwapperError {
    let lower = error.message.to_ascii_lowercase();
    if lower.contains("too low") {
        SwapperError::InputAmountError {
            min_amount: parse_min_amount(&error.message, from_decimals),
        }
    } else {
        SwapperError::ComputeQuoteError(format!("Near Intents quote error: {}", error.message))
    }
}

fn parse_min_amount(message: &str, decimals: u32) -> Option<String> {
    let marker = "try at least ";
    let lower = message.to_ascii_lowercase();
    let start = lower.find(marker)? + marker.len();
    let tail = message.get(start..)?;
    let token = extract_numeric_token(tail)?;
    amount_to_value(&token, decimals)
}

fn extract_numeric_token(message: &str) -> Option<String> {
    let mut current = String::new();

    for ch in message.chars() {
        if ch.is_ascii_digit() || ch == '.' || ch == ',' || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            return Some(current);
        }
    }

    if current.is_empty() { None } else { Some(current) }
}

#[async_trait]
impl<C> Swapper for NearIntents<C>
where
    C: gem_client::Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        self.supported_assets.clone()
    }

    fn amount_mode(&self, request: &QuoteRequest) -> SwapAmountMode {
        chain::swap_type(request.from_asset.chain()).amount_mode()
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let quote_request = Self::build_quote_request(request, true)?;
        let response = Self::extract_quote(self.client.get_quote(&quote_request).await?, request.from_asset.decimals)?;

        let eta = response.quote.time_estimate;
        let min_amount_in = BigUint::from_str(&response.quote.min_amount_in.to_string()).map_err(SwapperError::compute_quote_error)?;
        let amount_out = BigUint::from_str(&response.quote.amount_out.to_string()).map_err(SwapperError::compute_quote_error)?;
        let route_data = serde_json::to_string(&quote_request)?;

        Ok(Quote {
            from_value: match quote_request.swap_type {
                SwapType::ExactInput => response.quote.amount_in,
                SwapType::FlexInput => request.value.clone(),
            },
            min_from_value: Some(min_amount_in),
            to_value: amount_out,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data,
                }],
            },
            request: request.clone(),
            eta_in_seconds: Some(eta),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let mut quote_request: NearQuoteRequest = serde_json::from_str(&route.route_data)?;
        let request_deposit_mode = quote_request.deposit_mode.clone();
        quote_request.dry = false;

        let response: QuoteResponse = Self::extract_quote(self.client.get_quote(&quote_request).await?, quote.request.from_asset.decimals)?;
        let QuoteResponse { quote_request: _, quote: near_quote } = response;

        let deposit_address = near_quote.deposit_address.ok_or_else(|| SwapperError::ComputeQuoteError("Missing depositAddress in Near Intents response".into()))?;
        let amount_in = near_quote.amount_in.to_string();
        let deposit_mode = near_quote
            .deposit_mode
            .or(Some(request_deposit_mode))
            .ok_or_else(|| SwapperError::ComputeQuoteError("Near Intents response missing deposit mode".into()))?;
        let from_asset = &quote.request.from_asset;

        let memo_required = deposit_memo_chains().contains(&from_asset.asset_id().chain);
        let deposit_memo = near_quote.deposit_memo.non_empty();

        if memo_required && deposit_mode != DepositMode::Memo {
            return Err(SwapperError::ComputeQuoteError("Near Intents Stellar deposits require a memo".into()));
        }
        if memo_required && deposit_memo.is_none() {
            return Err(SwapperError::ComputeQuoteError("Near Intents Stellar deposit missing memo".into()));
        }

        let data = self.build_deposit_data(deposit_memo, from_asset, &quote.request.wallet_address, &deposit_address, &amount_in).await?;

        let DepositData { to, value, data: payload, memo } = data;

        Ok(SwapperQuoteData {
            data: payload,
            ..SwapperQuoteData::new_transfer(to, BigUint::from_str(&value).map_err(SwapperError::compute_quote_error)?, memo)
        })
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        let transaction = match (self.explorer.search_transaction(&request.transaction_hash).await?, &request.deposit_address) {
            (Some(transaction), _) => Some(transaction),
            (None, Some(deposit_address)) => self.explorer.search_deposit(deposit_address, request.deposit_memo.as_deref()).await?,
            (None, None) => None,
        };
        let Some(transaction) = transaction else {
            return Ok(SwapResult::pending());
        };

        let status = Self::map_transaction_status(&transaction.status);
        let metadata = Self::build_swap_metadata(&transaction, &status);

        Ok(SwapResult { status, metadata, eta_in_seconds: None })
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses {
            deposit: vec![],
            send: TREASURY_ADDRESSES.iter().map(ToString::to_string).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperError, SwapperQuoteAsset, alien::mock::ProviderMock, near_intents::testkit::mock_hypercore_quote_request};
    use primitives::{
        AssetId, Chain,
        asset_constants::{POLYGON_USDC_ASSET_ID, TON_USDT_ASSET_ID},
    };
    use serde_json::{Value, json};

    fn status(json: &str) -> SwapResult {
        let transactions: Vec<ExplorerTransaction> = serde_json::from_str(json).unwrap();
        let transaction = &transactions[0];
        let status = NearIntents::<RpcClient>::map_transaction_status(&transaction.status);
        let metadata = NearIntents::<RpcClient>::build_swap_metadata(transaction, &status);
        SwapResult { status, metadata, eta_in_seconds: None }
    }

    #[test]
    fn max_quote_keeps_transfer_amount() {
        let mut request = QuoteRequest::mock(Chain::Tron, None);
        request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Near));
        request.value = BigUint::from(37000000u64);
        request.options.use_max_amount = true;

        let quote_request = NearIntents::<RpcClient>::build_quote_request(&request, true).unwrap();

        assert_eq!(quote_request.amount, "37000000");
    }

    #[tokio::test]
    async fn test_hypercore_quote_data() {
        let provider = NearIntents::new(Arc::new(ProviderMock::new(include_str!("testdata/quote_hypercore_to_ethereum_usdc.json").to_string()))).unwrap();
        let quote = provider.get_quote(&mock_hypercore_quote_request()).await.unwrap();
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let payload: Value = serde_json::from_str(&data.data).unwrap();

        assert_eq!(quote.from_value, BigUint::from(10002430500_u64));
        assert_eq!(payload["message"]["amount"], "100.024305");
        assert_eq!(
            data,
            SwapperQuoteData {
                data: data.data.clone(),
                ..SwapperQuoteData::new_transfer("0x1085c5f70F7F7591D97da281A64688385455c2bD".into(), BigUint::from(10002430500_u64), None)
            }
        );
    }

    #[test]
    fn test_bitcoin_quote_deadline_is_two_hours() {
        for (from_chain, to_chain) in [(Chain::Bitcoin, Chain::Litecoin), (Chain::Litecoin, Chain::Bitcoin)] {
            let mut request = QuoteRequest::mock(from_chain, None);
            request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(to_chain));
            let earliest_deadline = Utc::now() + Duration::hours(2);

            let quote_request = NearIntents::<RpcClient>::build_quote_request(&request, true).unwrap();
            let latest_deadline = Utc::now() + Duration::hours(2);
            let deadline: DateTime<Utc> = quote_request.deadline.parse().unwrap();

            assert!(deadline >= earliest_deadline);
            assert!(deadline <= latest_deadline);
        }
    }

    #[test]
    fn swap_result_avax_to_smartchain() {
        let result = status(include_str!("testdata/tx_status_avax_to_smartchain.json"));

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::AvalancheC),
                    BigUint::from(28000000000000000u64),
                    AssetId::from_chain(Chain::SmartChain),
                    BigUint::from(399605209991817u64),
                    SwapperProvider::NearIntents
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn swap_result_referral_fee() {
        let metadata = status(include_str!("testdata/tx_status_polygon_usdc_to_litecoin.json")).metadata.unwrap();

        assert_eq!(metadata.from_asset, POLYGON_USDC_ASSET_ID.clone());
        assert_eq!(metadata.from_value, BigUint::from(70000000u64));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Litecoin));
        assert_eq!(metadata.to_value, BigUint::from(101438912u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: POLYGON_USDC_ASSET_ID.clone(),
                value: BigUint::from(175000u64),
            })
        );
    }

    #[test]
    fn swap_result_solana_to_bitcoin() {
        let result = status(include_str!("testdata/tx_status_solana_to_bitcoin.json"));

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Pending,
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Solana),
                    BigUint::from(646605458u64),
                    AssetId::from_chain(Chain::Bitcoin),
                    BigUint::from(69086u64),
                    SwapperProvider::NearIntents
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn swap_result_pending_deposit_without_amounts() {
        let result = status(include_str!("testdata/tx_status_solana_to_arbitrum_pending_deposit.json"));

        assert_eq!(result, SwapResult::pending());
    }

    #[test]
    fn swap_result_ton_to_smartchain_refunded() {
        let result = status(include_str!("testdata/tx_status_ton_to_smartchain_refunded.json"));

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Refunded,
                metadata: Some(TransactionSwapMetadata::new(
                    TON_USDT_ASSET_ID.clone(),
                    BigUint::from(6321766u64),
                    AssetId::from_chain(Chain::SmartChain),
                    BigUint::from(9690124016594003u64),
                    SwapperProvider::NearIntents
                )),
                eta_in_seconds: None,
            }
        );
    }

    #[test]
    fn swap_result_refunded_without_referral_fee() {
        let result = status(include_str!("testdata/tx_status_tron_to_litecoin_refunded_app_fee.json"));

        assert_eq!(result.status, SwapStatus::Refunded);
        assert_eq!(result.metadata.unwrap().referral_fee, None);
    }

    #[test]
    fn map_transaction_status_values() {
        let map = NearIntents::<RpcClient>::map_transaction_status;

        assert_eq!(map("SUCCESS"), SwapStatus::Completed);
        assert_eq!(map("SWAP_COMPLETED"), SwapStatus::Completed);
        assert_eq!(map("SWAP_COMPLETED_TX"), SwapStatus::Completed);

        assert_eq!(map("FAILED"), SwapStatus::Failed);
        assert_eq!(map("SWAP_FAILED"), SwapStatus::Failed);
        assert_eq!(map("REFUNDED"), SwapStatus::Refunded);
        assert_eq!(map("SWAP_REFUNDED"), SwapStatus::Refunded);
        assert_eq!(map("SWAP_LIQUIDITY_TIMEOUT"), SwapStatus::Failed);
        assert_eq!(map("SWAP_RISK_FAILED"), SwapStatus::Failed);

        assert_eq!(map("PENDING_DEPOSIT"), SwapStatus::Pending);
        assert_eq!(map("PROCESSING"), SwapStatus::Pending);
        assert_eq!(map("KNOWN_DEPOSIT_TX"), SwapStatus::Pending);
        assert_eq!(map("INCOMPLETE_DEPOSIT"), SwapStatus::Pending);
        assert_eq!(map("UNKNOWN_STATUS"), SwapStatus::Pending);
    }

    #[test]
    fn decode_quote_response_error_message() {
        let payload = json!({
            "message": "Amount is too low for bridge, try at least 8516130",
        });

        let decoded: QuoteResponseResult = serde_json::from_value(payload).unwrap();

        let QuoteResponseResult::Err(err) = decoded else {
            panic!("expected error variant");
        };
        assert_eq!(err.message, "Amount is too low for bridge, try at least 8516130");
        assert_eq!(map_quote_error(&err, 6), SwapperError::InputAmountError { min_amount: Some("8516130".into()) });
    }

    #[test]
    fn test_validate_minimum_amount() {
        assert_eq!(validate_minimum_amount(&100_u32.into(), &100_u32.into(), &90_u32.into(), &10_u32.into()), Ok(()));
        assert_eq!(
            validate_minimum_amount(&100_u32.into(), &100_u32.into(), &90_u32.into(), &11_u32.into()),
            Err(SwapperError::InputAmountError { min_amount: Some("110".into()) })
        );
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use super::*;
    use crate::near_intents::assets::NEAR_INTENTS_BTC_NATIVE;
    use crate::near_intents::testkit::mock_hypercore_quote_request;
    use crate::{FetchQuoteData, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARBITRUM_USDC_ASSET_ID, BASE_USDC_ASSET_ID, NEAR_USDT_ASSET_ID},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_near_intents_hypercore_quote() -> Result<(), SwapperError> {
        let provider = NearIntents::new(Arc::new(NativeProvider::new().set_debug(true))).unwrap();
        let request = mock_hypercore_quote_request();

        let quote = provider.get_quote(&request).await?;
        assert!(quote.from_value <= request.value);
        assert!(quote.to_value > BigUint::ZERO);
        Ok(())
    }

    #[tokio::test]
    async fn test_near_intents_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider).unwrap();

        let options = Options::mock_exact(100);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(ARBITRUM_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(500000u64),
            options,
        };

        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_near_intents_near_to_usdt_quote() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Near)),
            to_asset: SwapperQuoteAsset::from(NEAR_USDT_ASSET_ID.clone()),
            wallet_address: "test.near".to_string(),
            destination_address: "test.near".to_string(),
            value: BigUint::parse_bytes(b"1000000000000000000000000", 10).unwrap(),
            options: Options::mock_exact(100),
        };

        let quote = provider.get_quote(&request).await?;

        assert!(quote.to_value > BigUint::ZERO);
        Ok(())
    }

    #[tokio::test]
    async fn test_near_intents_bitcoin_quotes() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider).unwrap();

        let from_bitcoin_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(100000u64),
            options: Options::mock_exact(100),
        };

        let quote = provider.get_quote(&from_bitcoin_request).await?;
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let quote_request: NearQuoteRequest = serde_json::from_str(&route.route_data)?;

        assert_eq!(quote_request.origin_asset, NEAR_INTENTS_BTC_NATIVE);
        assert!(quote.to_value > BigUint::ZERO);

        println!("Near Intents BTC quote: from_value={}, to_value={}, eta={:?}", quote.from_value, quote.to_value, quote.eta_in_seconds);
        println!("Near Intents BTC quote request: {}", route.route_data);

        let to_bitcoin_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            value: BigUint::from(10000000u64),
            options: Options::mock_exact(100),
        };

        let quote = provider.get_quote(&to_bitcoin_request).await?;
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let quote_request: NearQuoteRequest = serde_json::from_str(&route.route_data)?;

        assert_eq!(quote_request.destination_asset, NEAR_INTENTS_BTC_NATIVE);
        assert!(quote.to_value > BigUint::ZERO);

        println!("Near Intents to BTC quote: from_value={}, to_value={}, eta={:?}", quote.from_value, quote.to_value, quote.eta_in_seconds);
        println!("Near Intents to BTC quote request: {}", route.route_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_near_intents_stellar_requires_memo() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider).unwrap();

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Stellar)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Near)),
            wallet_address: "GBZXN7PIRZGNMHGA3RSSOEV56YXG54FSNTJDGQI3GHDVBKSXRZ5B6KJT".to_string(),
            destination_address: "test.near".to_string(),
            value: BigUint::from(20000000u64),
            options: Options::mock_exact(100),
        };

        let quote = match provider.get_quote(&request).await {
            Ok(quote) => quote,
            Err(SwapperError::ComputeQuoteError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };
        let quote_data = match provider.get_quote_data(&quote, FetchQuoteData::None).await {
            Ok(data) => data,
            Err(SwapperError::TransactionError(_)) => return Ok(()),
            Err(error) => return Err(error),
        };

        assert!(!quote_data.data.is_empty(), "expected deposit memo for Stellar swaps via Near Intents");

        Ok(())
    }

    #[tokio::test]
    async fn test_near_intents_status() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::new().set_debug(true));
        let provider = NearIntents::new(rpc_provider).unwrap();
        let deposit_address = "18gB9wZz1Q4CzniurLye1KdUUqjWjo3ePr";

        let request = SwapResultRequest {
            deposit_address: Some(deposit_address.to_string()),
            ..SwapResultRequest::new(Chain::Bitcoin, "")
        };
        let swap_result = provider.get_swap_result(&request).await?;

        println!("swap_result: {swap_result:?}");

        Ok(())
    }
}
