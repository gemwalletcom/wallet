use super::{
    DEFAULT_FILL_TIMEOUT,
    asset::{across_asset_id, parse_address},
    config_store::{ConfigStoreClient, TokenConfig},
    hubpool::HubPoolClient,
};
use crate::{
    SwapAmountMode, SwapResult, Swapper, SwapperError, SwapperProvider, SwapperQuoteData,
    across::{DEFAULT_DEPOSIT_GAS_LIMIT, DEFAULT_FILL_GAS_LIMIT},
    alien::RpcProvider,
    approval::{check_approval_erc20, check_approval_trc20, get_swap_gas_limit_with_approval},
    chainlink::ChainlinkPriceFeed,
    client_factory::create_eth_client,
    cross_chain::VaultAddresses,
    eth_address,
    fees::{ReferralFee, default_referral_fees},
    models::*,
};
use alloy_primitives::{Address, Bytes, U256, hex::decode as HexDecode, hex::encode_prefixed as HexEncode};
use alloy_sol_types::{SolCall, SolValue};
use async_trait::async_trait;
use gem_evm::{
    across::{
        contracts::{
            V3SpokePoolInterface::{self, V3RelayData},
            multicall_handler,
        },
        deployment::AcrossDeployment,
        fees::{self, LpFeeCalculator, RateModel, RelayerFeeCalculator},
    },
    contracts::erc20::IERC20,
    jsonrpc::TransactionObject,
    multicall3::IMulticall3,
    u256::biguint_to_u256,
    weth::WETH9,
};
use num_bigint::{BigInt, Sign};
use primitives::{AssetId, Chain, EVMChain, swap::ApprovalData};
use serde_serializers::biguint_from_hex_str;
use std::{fmt::Debug, str::FromStr, sync::Arc};

#[derive(Debug)]
pub struct Across {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Across {
    fn bigint_to_u256(value: &BigInt) -> Result<U256, SwapperError> {
        if value.sign() == Sign::Minus {
            return Err(SwapperError::ComputeQuoteError("Negative value provided for gas computation".into()));
        }
        biguint_to_u256(value.magnitude()).ok_or_else(|| SwapperError::ComputeQuoteError("Gas value exceeds U256".into()))
    }

    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Across),
            rpc_provider,
        }
    }

    pub fn boxed(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
        Box::new(Self::new(rpc_provider))
    }

    fn is_supported_route(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        if to_asset.chain == Chain::Tron {
            return false;
        }

        let Some(from) = across_asset_id(from_asset) else {
            return false;
        };
        let Some(to) = across_asset_id(to_asset) else {
            return false;
        };

        AcrossDeployment::asset_mappings().into_iter().any(|x| x.set.contains(&from) && x.set.contains(&to))
    }

    fn get_rate_model(from_asset: &AssetId, to_asset: &AssetId, token_config: &TokenConfig) -> RateModel {
        let key = format!("{}-{}", from_asset.chain.network_id(), to_asset.chain.network_id());
        let rate_model = token_config.route_rate_model.get(&key).unwrap_or(&token_config.rate_model);
        rate_model.clone().into()
    }

    async fn gas_price(&self, chain: Chain) -> Result<U256, SwapperError> {
        let gas_price = create_eth_client(self.rpc_provider.clone(), chain)?.gas_price().await?;
        Self::bigint_to_u256(&gas_price)
    }

    async fn multicall3(&self, chain: Chain, calls: Vec<IMulticall3::Call3>) -> Result<Vec<IMulticall3::Result>, SwapperError> {
        create_eth_client(self.rpc_provider.clone(), chain)?
            .multicall3(calls)
            .await
            .map_err(SwapperError::compute_quote_error)
    }

    async fn estimate_gas_transaction(&self, chain: Chain, tx: TransactionObject) -> Result<U256, SwapperError> {
        let client = create_eth_client(self.rpc_provider.clone(), chain)?;
        let gas_hex = client.estimate_gas(tx.from.as_deref(), &tx.to, tx.value.as_deref(), Some(tx.data.as_str())).await?;

        let gas_biguint = biguint_from_hex_str(&gas_hex).map_err(|e| SwapperError::ComputeQuoteError(format!("Failed to parse gas estimate: {e}")))?;
        let gas_bigint = BigInt::from_biguint(Sign::Plus, gas_biguint);
        Self::bigint_to_u256(&gas_bigint)
    }

    /// Return (message, referral_fee)
    fn message_for_multicall_handler(
        &self,
        amount: &U256,
        original_output_asset: &AssetId,
        output_token: &Address,
        user_address: &Address,
        output_chain: Chain,
        referral_fee: &ReferralFee,
    ) -> Result<(Vec<u8>, U256), SwapperError> {
        if referral_fee.bps == 0 {
            return Ok((vec![], U256::from(0)));
        }
        let fee_address = parse_address(output_chain, &referral_fee.address)?;
        let fee_amount = amount * U256::from(referral_fee.bps) / U256::from(10000);
        let user_amount = amount - fee_amount;

        let calls = if original_output_asset.is_native() {
            // output_token is WETH and we need to unwrap it
            Self::unwrap_weth_calls(output_token, amount, user_address, &user_amount, &fee_address, &fee_amount)
        } else {
            Self::erc20_transfer_calls(output_token, user_address, &user_amount, &fee_address, &fee_amount)
        };
        let instructions = multicall_handler::Instructions {
            calls,
            fallbackRecipient: *user_address,
        };
        let message = instructions.abi_encode();
        Ok((message, fee_amount))
    }

    fn unwrap_weth_calls(
        weth_contract: &Address,
        output_amount: &U256,
        user_address: &Address,
        user_amount: &U256,
        fee_address: &Address,
        fee_amount: &U256,
    ) -> Vec<multicall_handler::Call> {
        assert!(fee_amount + user_amount == *output_amount);
        let withdraw_call = WETH9::withdrawCall { wad: *output_amount };
        vec![
            multicall_handler::Call {
                target: *weth_contract,
                callData: withdraw_call.abi_encode().into(),
                value: U256::from(0),
            },
            multicall_handler::Call {
                target: *user_address,
                callData: Bytes::new(),
                value: *user_amount,
            },
            multicall_handler::Call {
                target: *fee_address,
                callData: Bytes::new(),
                value: *fee_amount,
            },
        ]
    }

    fn erc20_transfer_calls(token: &Address, user_address: &Address, user_amount: &U256, fee_address: &Address, fee_amount: &U256) -> Vec<multicall_handler::Call> {
        let target = *token;
        let user_transfer = IERC20::transferCall {
            to: *user_address,
            value: *user_amount,
        };
        let fee_transfer = IERC20::transferCall {
            to: *fee_address,
            value: *fee_amount,
        };
        vec![
            multicall_handler::Call {
                target,
                callData: user_transfer.abi_encode().into(),
                value: U256::from(0),
            },
            multicall_handler::Call {
                target,
                callData: fee_transfer.abi_encode().into(),
                value: U256::from(0),
            },
        ]
    }

    async fn estimate_gas_limit(
        &self,
        amount: &U256,
        is_native: bool,
        input_asset: &AssetId,
        output_token: &Address,
        depositor: &Address,
        recipient: &Address,
        message: &[u8],
        deployment: &AcrossDeployment,
        chain: Chain,
    ) -> Result<(U256, V3RelayData), SwapperError> {
        let chain_id = deployment.chain_id;

        let recipient = if message.is_empty() {
            *recipient
        } else {
            parse_address(chain, deployment.multicall_handler().as_str())?
        };

        let v3_relay_data = V3RelayData {
            depositor: *depositor,
            recipient,
            exclusiveRelayer: Address::ZERO,
            inputToken: parse_address(input_asset.chain, input_asset.token_id.as_deref().ok_or(SwapperError::NotSupportedAsset)?)?,
            outputToken: *output_token,
            inputAmount: *amount,
            outputAmount: U256::from(100),
            originChainId: U256::from(chain_id),
            depositId: u32::MAX,
            fillDeadline: u32::MAX,
            exclusivityDeadline: 0,
            message: Bytes::from(message.to_vec()),
        };
        let value = if is_native { format!("{amount:#x}") } else { String::from("0x0") };
        let data = V3SpokePoolInterface::fillV3RelayCall {
            relayData: v3_relay_data.clone(),
            repaymentChainId: U256::from(chain_id),
        }
        .abi_encode();
        let tx = TransactionObject::new_call_to_value(deployment.spoke_pool, &value, data);
        let default_fill_limit = match chain {
            Chain::Monad => DEFAULT_FILL_GAS_LIMIT * 3,
            _ => DEFAULT_FILL_GAS_LIMIT,
        };
        let gas_limit = self.estimate_gas_transaction(chain, tx).await.unwrap_or(U256::from(default_fill_limit));
        Ok((gas_limit, v3_relay_data))
    }

    async fn usd_price_for_chain(&self, chain: Chain, existing_results: &[IMulticall3::Result]) -> Result<BigInt, SwapperError> {
        let feed = ChainlinkPriceFeed::new_usd_feed_for_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        if chain == Chain::Monad {
            let results = create_eth_client(self.rpc_provider.clone(), Chain::Monad)?
                .multicall3(vec![feed.latest_round_call3()])
                .await
                .map_err(SwapperError::compute_quote_error)?;
            ChainlinkPriceFeed::decoded_answer(&results[0])
        } else {
            ChainlinkPriceFeed::decoded_answer(&existing_results[3])
        }
    }

    fn calculate_fee_in_token(fee_in_wei: &U256, token_price: &BigInt, token_decimals: u32) -> U256 {
        let fee = BigInt::from_bytes_le(Sign::Plus, &fee_in_wei.to_le_bytes::<32>());
        let fee_in_token = fee * token_price * BigInt::from(10_u64.pow(token_decimals)) / BigInt::from(10_u64.pow(8)) / BigInt::from(10_u64.pow(18));
        U256::from_le_slice(&fee_in_token.to_bytes_le().1)
    }

    fn get_eta_in_seconds(&self, from_chain: &Chain, to_chain: &Chain) -> Option<u32> {
        let from_chain = EVMChain::from_chain(*from_chain)?;
        let to_chain = EVMChain::from_chain(*to_chain)?;
        let from_chain_l2 = from_chain.is_ethereum_layer2();
        let to_chain_l2 = to_chain.is_ethereum_layer2();
        Some(match (from_chain_l2, to_chain_l2) {
            (true, true) => 5,   // L2 to L2
            (true, false) => 10, // L2 to L1
            (false, _) => 20,    // L1 to L2
        })
    }
}

#[async_trait]
impl Swapper for Across {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        let supported_assets = AcrossDeployment::supported_assets();
        Chain::all()
            .into_iter()
            .filter_map(|chain| supported_assets.get(&chain).map(|assets| SwapperChainAsset::Assets(chain, assets.clone())))
            .collect()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        if request.from_asset.chain() == request.to_asset.chain() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input_is_native = request.from_asset.is_native();
        let from_chain = request.from_asset.chain();
        if from_chain == Chain::Tron && input_is_native {
            return Err(SwapperError::NotSupportedAsset);
        }
        let from_amount: U256 = request.value.parse().map_err(SwapperError::from)?;
        let depositor_address = parse_address(from_chain, &request.wallet_address)?;
        let recipient_address = parse_address(request.to_asset.chain(), &request.destination_address)?;

        AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_route(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input_asset = across_asset_id(&request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let output_asset = across_asset_id(&request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let original_output_asset = request.to_asset.asset_id();
        let output_token = parse_address(output_asset.chain, output_asset.token_id.as_deref().ok_or(SwapperError::NotSupportedAsset)?)?;

        // Get L1 token address
        let mappings = AcrossDeployment::asset_mappings();
        let asset_mapping = mappings.iter().find(|x| x.set.contains(&input_asset)).ok_or(SwapperError::NotSupportedAsset)?;
        let asset_mainnet = asset_mapping.set.iter().find(|x| x.chain == Chain::Ethereum).ok_or(SwapperError::NotSupportedAsset)?;
        let mainnet_token = eth_address::parse_asset_id(asset_mainnet)?;

        let hubpool_client = HubPoolClient::new();
        let config_client = ConfigStoreClient::new(self.rpc_provider.clone());

        let calls = vec![
            hubpool_client.paused_call3(),
            hubpool_client.sync_call3(&mainnet_token),
            hubpool_client.pooled_token_call3(&mainnet_token),
        ];
        let results = self.multicall3(Chain::Ethereum, calls).await?;

        // Check if protocol is paused
        let is_paused = hubpool_client.decoded_paused_call3(&results[0])?;
        if is_paused {
            return Err(SwapperError::ComputeQuoteError("Across protocol is paused".into()));
        }

        // Check bridge amount is too large (Across API has some limit in USD amount but we don't have that info)
        if from_amount > hubpool_client.decoded_pooled_token_call3(&results[2])?.liquidReserves {
            return Err(SwapperError::ComputeQuoteError("Bridge amount is too large".into()));
        }

        // Prepare data for lp fee calculation (token config, utilization, current time)
        let token_config_req = config_client.fetch_config(&mainnet_token);
        let mut calls = vec![
            hubpool_client.utilization_call3(&mainnet_token, U256::from(0)),
            hubpool_client.utilization_call3(&mainnet_token, from_amount),
            hubpool_client.get_current_time(),
        ];

        if !original_output_asset.is_native() {
            let gas_price_feed = ChainlinkPriceFeed::new_usd_feed_for_chain(request.to_asset.chain()).unwrap_or_else(ChainlinkPriceFeed::new_eth_usd_feed);
            calls.push(gas_price_feed.latest_round_call3());
        }

        let multicall_results = self.multicall3(Chain::Ethereum, calls).await?;
        let token_config = token_config_req.await?;

        let util_before = hubpool_client.decoded_utilization_call3(&multicall_results[0])?;
        let util_after = hubpool_client.decoded_utilization_call3(&multicall_results[1])?;
        let timestamp = hubpool_client.decoded_current_time(&multicall_results[2])?;

        let rate_model = Self::get_rate_model(&input_asset, &output_asset, &token_config);
        let cost_config = &asset_mapping.capital_cost;

        // Calculate lp fee
        let lpfee_calc = LpFeeCalculator::new(rate_model);
        let lpfee_percent = lpfee_calc.realized_lp_fee_pct(&util_before, &util_after, false);
        let lpfee = fees::multiply(from_amount, lpfee_percent, cost_config.decimals);

        // Calculate relayer fee
        let relayer_calc = RelayerFeeCalculator::default();
        let relayer_fee_percent = relayer_calc.capital_fee_percent(&BigInt::from_str(&request.value)?, cost_config);
        let relayer_fee = fees::multiply(from_amount, relayer_fee_percent, cost_config.decimals);

        let referral_config = default_referral_fees().for_chain(request.to_asset.chain()).cloned().unwrap_or_default();

        // Calculate gas limit / price for relayer
        let remain_amount = from_amount - lpfee - relayer_fee;
        let (message, referral_fee) = self.message_for_multicall_handler(
            &remain_amount,
            &original_output_asset,
            &output_token,
            &recipient_address,
            request.to_asset.chain(),
            &referral_config,
        )?;

        let (gas_limit, mut v3_relay_data) = self
            .estimate_gas_limit(
                &from_amount,
                input_is_native,
                &input_asset,
                &output_token,
                &depositor_address,
                &recipient_address,
                &message,
                &destination_deployment,
                request.to_asset.chain(),
            )
            .await?;
        let native_gas_fee = gas_limit * self.gas_price(request.to_asset.chain()).await?;
        let gas_fee = if original_output_asset.is_native() {
            native_gas_fee
        } else {
            let price = self.usd_price_for_chain(request.to_asset.chain(), &multicall_results).await?;
            Self::calculate_fee_in_token(&native_gas_fee, &price, cost_config.decimals)
        };

        // Check if bridge amount is too small
        if remain_amount < gas_fee {
            return Err(SwapperError::InputAmountError { min_amount: None });
        }

        let output_amount = remain_amount - gas_fee;
        let to_value = output_amount - referral_fee;

        // Update v3 relay data (was used to estimate gas limit) with final output amount, quote timestamp and referral fee.
        let (message, _) = self.message_for_multicall_handler(
            &output_amount,
            &original_output_asset,
            &output_token,
            &recipient_address,
            request.to_asset.chain(),
            &referral_config,
        )?;
        v3_relay_data.outputAmount = output_amount;
        v3_relay_data.fillDeadline = timestamp + DEFAULT_FILL_TIMEOUT;
        v3_relay_data.message = message.into();
        let route_data = HexEncode(v3_relay_data.abi_encode());

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: input_asset.clone(),
                    output: output_asset.clone(),
                    route_data,
                }],
            },
            request: request.clone(),
            eta_in_seconds: self.get_eta_in_seconds(&request.from_asset.chain(), &request.to_asset.chain()),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_chain = quote.request.from_asset.chain();
        let deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&quote.request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let route = &quote.data.routes[0];
        let route_data = HexDecode(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let deposit_v3_call = V3SpokePoolInterface::depositV3Call {
            depositor: v3_relay_data.depositor,
            recipient: v3_relay_data.recipient,
            inputToken: v3_relay_data.inputToken,
            outputToken: v3_relay_data.outputToken,
            inputAmount: v3_relay_data.inputAmount,
            outputAmount: v3_relay_data.outputAmount,
            destinationChainId: U256::from(destination_deployment.chain_id),
            exclusiveRelayer: Address::ZERO,
            quoteTimestamp: v3_relay_data.fillDeadline - DEFAULT_FILL_TIMEOUT,
            fillDeadline: v3_relay_data.fillDeadline,
            exclusivityDeadline: 0,
            message: v3_relay_data.message,
        }
        .abi_encode();

        let input_is_native = quote.request.from_asset.is_native();
        let value: &str = if input_is_native { &quote.from_value } else { "0" };

        let approval: Option<ApprovalData> = {
            if input_is_native {
                None
            } else if from_chain == Chain::Tron {
                check_approval_trc20(
                    quote.request.wallet_address.clone(),
                    quote.request.from_asset.asset_id().token_id.ok_or(SwapperError::NotSupportedAsset)?,
                    deployment.spoke_pool.into(),
                    v3_relay_data.inputAmount,
                    self.rpc_provider.clone(),
                )
                .await?
                .approval_data()
            } else {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    v3_relay_data.inputToken.to_string(),
                    deployment.spoke_pool.into(),
                    v3_relay_data.inputAmount,
                    self.rpc_provider.clone(),
                    &from_chain,
                )
                .await?
                .approval_data()
            }
        };

        let is_tron = from_chain == Chain::Tron;
        let to: String = deployment.spoke_pool.into();
        let mut gas_limit = if is_tron {
            None
        } else {
            get_swap_gas_limit_with_approval(&approval, None, DEFAULT_DEPOSIT_GAS_LIMIT)
        };

        let should_estimate_gas = matches!(data, FetchQuoteData::EstimateGas) && !is_tron;
        if should_estimate_gas {
            let hex_value = format!("{:#x}", U256::from_str(value)?);
            let tx = TransactionObject::new_call_to_value(&to, &hex_value, deposit_v3_call.clone());
            gas_limit = Some(self.estimate_gas_transaction(from_chain, tx).await?.to_string());
        }

        Ok(SwapperQuoteData::new_contract(
            deployment.spoke_pool.into(),
            value.to_string(),
            HexEncode(deposit_v3_call.clone()),
            approval,
            gas_limit,
        ))
    }
    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses {
            deposit: AcrossDeployment::deposit_addresses(),
            send: AcrossDeployment::send_addresses(),
        })
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        super::status::get_swap_result(self.rpc_provider.clone(), chain, transaction_hash).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::mock::{MockFn, ProviderMock};
    use gem_evm::multicall3::IMulticall3;
    use primitives::{asset_constants::*, swap::SwapQuoteDataType};
    use std::time::Duration;

    const TEST_FILL_DEADLINE: u32 = 1_700_000_000 + DEFAULT_FILL_TIMEOUT;

    #[test]
    fn test_is_supported_route() {
        let weth_eth: AssetId = ETHEREUM_WETH_ASSET_ID.clone();
        let weth_op: AssetId = OPTIMISM_WETH_ASSET_ID.clone();
        let weth_arb: AssetId = ARBITRUM_WETH_ASSET_ID.clone();
        let weth_blast: AssetId = BLAST_WETH_ASSET_ID.clone();
        let weth_bsc: AssetId = SMARTCHAIN_ETH_ASSET_ID.clone();

        let usdc_eth: AssetId = ETHEREUM_USDC_ASSET_ID.clone();
        let usdc_arb: AssetId = ARBITRUM_USDC_ASSET_ID.clone();
        let usdc_monad: AssetId = MONAD_USDC_ASSET_ID.clone();
        let usdt_eth: AssetId = ETHEREUM_USDT_ASSET_ID.clone();
        let usdt_monad: AssetId = MONAD_USDT_ASSET_ID.clone();
        let usdt_tron: AssetId = TRON_USDT_ASSET_ID.clone();

        assert!(Across::is_supported_route(&weth_eth, &weth_op));
        assert!(Across::is_supported_route(&weth_op, &weth_arb));
        assert!(!Across::is_supported_route(&weth_eth, &weth_blast));
        assert!(!Across::is_supported_route(&weth_blast, &weth_eth));
        assert!(Across::is_supported_route(&usdc_eth, &usdc_arb));
        assert!(Across::is_supported_route(&usdc_monad, &usdc_eth));
        assert!(Across::is_supported_route(&usdt_monad, &usdt_eth));
        assert!(Across::is_supported_route(&usdt_tron, &usdt_eth));
        assert!(!Across::is_supported_route(&usdt_eth, &usdt_tron));
        assert!(Across::is_supported_route(&weth_eth, &weth_bsc));

        assert!(!Across::is_supported_route(&weth_eth, &usdc_eth));
        assert!(!Across::is_supported_route(&weth_eth, &usdt_tron));

        // native asset
        let eth = AssetId::from(Chain::Ethereum, None);
        let op = AssetId::from(Chain::Optimism, None);
        let arb = AssetId::from(Chain::Arbitrum, None);
        let linea = AssetId::from(Chain::Linea, None);

        assert!(Across::is_supported_route(&eth, &linea));
        assert!(Across::is_supported_route(&op, &eth));
        assert!(Across::is_supported_route(&arb, &eth));
        assert!(Across::is_supported_route(&op, &arb));
    }

    fn provider_with_tron_allowance(allowance: &str) -> Across {
        let response = format!(r#"{{"constant_result":["{allowance}"]}}"#);
        Across::new(Arc::new(ProviderMock {
            response: MockFn(Box::new(move |_| response.clone())),
            timeout: Duration::from_millis(10),
        }))
    }

    fn tron_quote(provider: &Across) -> (Quote, V3RelayData) {
        let relay_data = V3RelayData {
            depositor: parse_address(Chain::Tron, "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC").unwrap(),
            recipient: eth_address::parse_str("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap(),
            exclusiveRelayer: Address::ZERO,
            inputToken: parse_address(Chain::Tron, TRON_USDT_TOKEN_ID).unwrap(),
            outputToken: eth_address::parse_str(ETHEREUM_USDT_TOKEN_ID).unwrap(),
            inputAmount: U256::from(10_000_000),
            outputAmount: U256::from(9_990_000),
            originChainId: U256::from(AcrossDeployment::deployment_by_chain(&Chain::Tron).unwrap().chain_id),
            depositId: u32::MAX,
            fillDeadline: TEST_FILL_DEADLINE,
            exclusivityDeadline: 0,
            message: Bytes::new(),
        };
        let request = QuoteRequest {
            from_asset: TRON_USDT_ASSET_ID.clone().into(),
            to_asset: ETHEREUM_USDT_ASSET_ID.clone().into(),
            wallet_address: "TJRyWwFs9wTFGZg3JbrVriFbNfCug5tDeC".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "10000000".to_string(),
            options: Options::default(),
        };
        let quote = Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: "9990000".to_string(),
            data: ProviderData {
                provider: provider.provider().clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: TRON_USDT_ASSET_ID.clone(),
                    output: ETHEREUM_USDT_ASSET_ID.clone(),
                    route_data: HexEncode(relay_data.abi_encode()),
                }],
            },
            request,
            eta_in_seconds: Some(120),
        };
        (quote, relay_data)
    }

    #[tokio::test]
    async fn test_direct_quote_data_maps_tron_approval() {
        let provider = provider_with_tron_allowance("0");
        let (quote, relay_data) = tron_quote(&provider);
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let expected_data = V3SpokePoolInterface::depositV3Call {
            depositor: relay_data.depositor,
            recipient: relay_data.recipient,
            inputToken: relay_data.inputToken,
            outputToken: relay_data.outputToken,
            inputAmount: relay_data.inputAmount,
            outputAmount: relay_data.outputAmount,
            destinationChainId: U256::from(AcrossDeployment::deployment_by_chain(&Chain::Ethereum).unwrap().chain_id),
            exclusiveRelayer: Address::ZERO,
            quoteTimestamp: TEST_FILL_DEADLINE - DEFAULT_FILL_TIMEOUT,
            fillDeadline: TEST_FILL_DEADLINE,
            exclusivityDeadline: 0,
            message: Bytes::new(),
        }
        .abi_encode();

        assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);
        assert_eq!(quote_data.to, "TTbCVPfUZmPhrB9sYC8GKgGBQQEdZovkmS");
        assert_eq!(quote_data.value, "0");
        assert_eq!(quote_data.data, HexEncode(expected_data));
        assert_eq!(quote_data.gas_limit, None);
        assert_eq!(
            quote_data.approval,
            Some(ApprovalData {
                token: TRON_USDT_TOKEN_ID.to_string(),
                spender: "TTbCVPfUZmPhrB9sYC8GKgGBQQEdZovkmS".to_string(),
                value: "10000000".to_string(),
                is_unlimited: true,
            })
        );
    }

    #[tokio::test]
    async fn test_direct_quote_data_skips_tron_approval_when_allowance_covers_amount() {
        let provider = provider_with_tron_allowance("989680");
        let (quote, _) = tron_quote(&provider);
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.approval, None);
        assert_eq!(quote_data.gas_limit, None);
    }

    #[test]
    fn test_fee_in_token() {
        let data = HexDecode("0x00000000000000000000000000000000000000000000000700000000000013430000000000000000000000000000000000000000000000000000004e17511aea00000000000000000000000000000000000000000000000000000000677e57a600000000000000000000000000000000000000000000000000000000677e57bb0000000000000000000000000000000000000000000000070000000000001343").unwrap();
        let result = IMulticall3::Result {
            success: true,
            returnData: data.into(),
        };
        let price = ChainlinkPriceFeed::decoded_answer(&result).unwrap();

        assert_eq!(price, BigInt::from(335398640362_u64));

        let gas_fee = U256::from(1861602902696880_u64);
        let fee_in_token = Across::calculate_fee_in_token(&gas_fee, &price, 6);

        assert_eq!(fee_in_token.to_string(), "6243790");
    }

    #[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
    mod swap_integration_tests {
        use super::*;
        use crate::{FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError};
        use primitives::{AssetId, Chain, swap::SwapStatus};
        use std::{sync::Arc, time::SystemTime};

        #[tokio::test]
        async fn test_across_quote() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options {
                slippage: 100.into(),
                use_max_amount: false,
            };

            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Optimism).into(),
                to_asset: AssetId::from_chain(Chain::Arbitrum).into(),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                value: "20000000000000000".into(), // 0.02 ETH
                options,
            };

            let now = SystemTime::now();
            let quote = swap_provider.get_quote(&request).await?;
            let elapsed = SystemTime::now().duration_since(now).unwrap();

            println!("<== elapsed: {:?}", elapsed);
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value.parse::<u64>().unwrap() > 0);

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
            println!("<== quote_data: {:?}", quote_data);

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote_eth_usdc_to_monad_usdc() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options {
                slippage: 100.into(),
                use_max_amount: false,
            };

            let wallet = "0x9b1fe00135e0ff09389bfaeff0c8f299ec818d4a";
            let from_asset: AssetId = ETHEREUM_USDC_ASSET_ID.clone();
            let to_asset: AssetId = MONAD_USDC_ASSET_ID.clone();
            let request = QuoteRequest {
                from_asset: from_asset.into(),
                to_asset: to_asset.into(),
                wallet_address: wallet.into(),
                destination_address: wallet.into(),
                value: "50000000".into(), // 50 USDC
                options,
            };

            let now = SystemTime::now();
            let quote = swap_provider.get_quote(&request).await?;
            let elapsed = SystemTime::now().duration_since(now).unwrap();

            println!("<== elapsed: {:?}", elapsed);
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value.parse::<u64>().unwrap() > 0);

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            println!("<== quote_data: {:?}", quote_data);

            Ok(())
        }

        #[tokio::test]
        async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::new(network_provider.clone());

            let tx_hash = "0x0a970040a9885cf2c8a42df6fcdf02a1f3fe7db12079a35613a665a2ee64df49";
            let chain = Chain::Arbitrum;

            let result = swap_provider.get_swap_result(chain, tx_hash).await?;

            println!("Across swap result: {:?}", result);
            assert_eq!(result.status, SwapStatus::Completed);

            let metadata = result.metadata.unwrap();
            assert_eq!(metadata.provider, Some("across".to_string()));
            assert!(!metadata.from_value.is_empty());
            assert!(!metadata.to_value.is_empty());

            Ok(())
        }
    }
}
