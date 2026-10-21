use super::{DEFAULT_DEPOSIT_GAS_LIMIT, DEFAULT_FILL_TIMEOUT, GAS_TOKEN_DECIMALS, config_store::TokenConfig, hubpool::HubPoolClient};
use crate::{
    SwapAmountMode, SwapResult, Swapper, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::RpcProvider,
    approval::{check_token_approval, get_swap_gas_limit_with_approval},
    chainlink::{self, ONE_DOLLAR},
    client_factory::{create_client_with_chain, create_eth_client},
    cross_chain::{VaultAddresses, rescale_base_units},
    eth_address,
    fees::{BRIDGE_FEE_BPS, ReferralFee, bps_of, referral_fees},
    models::*,
};
use alloy_primitives::{Address, Bytes, U256, hex::decode as HexDecode, hex::encode_prefixed as HexEncode};
use alloy_sol_types::{SolCall, SolValue};
use async_trait::async_trait;
use futures::try_join;
use gem_evm::u256::u256_to_biguint;
use gem_evm::{
    across::{
        asset::{AcrossAsset, Funding},
        contracts::{
            V3SpokePoolInterface::{self, V3RelayData},
            multicall_handler,
        },
        deployment::{AcrossDeployment, FillCost, TokenPrice},
        fees::{self, LpFeeCalculator, RelayerFeeCalculator},
    },
    contracts::erc20::IERC20,
    jsonrpc::{EthereumRpc, TransactionObject},
    multicall3::{IMulticall3, deployment_by_chain},
    u256::biguint_to_u256,
    weth::WETH9,
};
use num_bigint::BigInt;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, EVMChain};
use serde_serializers::biguint_from_hex_str;
use std::{fmt::Debug, str::FromStr, sync::Arc};

struct HubPoolQuote {
    lp_fee_percent: BigInt,
    timestamp: u32,
}

#[derive(Debug)]
pub struct Across {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Across {
    fn gas_value(value: &str) -> Result<U256, SwapperError> {
        let value = biguint_from_hex_str(value).map_err(|error| SwapperError::ComputeQuoteError(format!("Failed to parse gas value: {error}")))?;
        biguint_to_u256(&value).ok_or_else(|| SwapperError::ComputeQuoteError("Gas value exceeds U256".into()))
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
        let supported_assets = AcrossDeployment::supported_assets();
        let is_declared = |asset: &AssetId| supported_assets.get(&asset.chain).is_some_and(|assets| asset.is_native() || assets.contains(asset));
        if !is_declared(from_asset) || !is_declared(to_asset) || AcrossDeployment::fill_cost(&to_asset.chain).is_none() {
            return false;
        }

        let (Some(from), Some(to)) = (AcrossAsset::from_asset(from_asset), AcrossAsset::from_asset(to_asset)) else {
            return false;
        };

        AcrossDeployment::asset_mappings().into_iter().any(|x| x.set.contains(&from.asset_id) && x.set.contains(&to.asset_id))
    }

    fn multicall_request(chain: Chain, calls: Vec<IMulticall3::Call3>) -> Result<EthereumRpc, SwapperError> {
        let chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let data = IMulticall3::aggregate3Call { calls }.abi_encode();
        Ok(EthereumRpc::call(deployment_by_chain(&chain), data))
    }

    fn decode_multicall(result: String) -> Result<Vec<IMulticall3::Result>, SwapperError> {
        let data = HexDecode(result).map_err(SwapperError::compute_quote_error)?;
        IMulticall3::aggregate3Call::abi_decode_returns(&data).map_err(SwapperError::from)
    }

    async fn estimate_gas_transaction(&self, chain: Chain, tx: TransactionObject) -> Result<U256, SwapperError> {
        let client = create_eth_client(self.rpc_provider.clone(), chain)?;
        let gas_hex = client.estimate_gas(tx.from.as_deref(), &tx.to, tx.value.as_deref(), Some(tx.data.as_str())).await?;
        Self::gas_value(&gas_hex)
    }

    async fn hub_pool_quote(&self, mainnet_token: &Address, amount: U256, input: &AssetId, output: &AssetId) -> Result<HubPoolQuote, SwapperError> {
        let hub_pool = HubPoolClient::new();
        let calls = vec![
            hub_pool.paused_call3(),
            hub_pool.sync_call3(mainnet_token),
            hub_pool.pooled_token_call3(mainnet_token),
            hub_pool.utilization_call3(mainnet_token, U256::ZERO),
            hub_pool.utilization_call3(mainnet_token, amount),
            hub_pool.get_current_time(),
        ];
        let requests = vec![Self::multicall_request(Chain::Ethereum, calls)?, TokenConfig::request(mainnet_token)];
        let [multicall_response, config_response]: [String; 2] = create_client_with_chain(self.rpc_provider.clone(), Chain::Ethereum)?
            .requests::<String, _>(requests)
            .await?
            .take_all()?
            .try_into()
            .map_err(|_| SwapperError::NoQuoteAvailable)?;
        let results = Self::decode_multicall(multicall_response)?;
        let [paused, _, pooled_token, utilization_before, utilization_after, current_time] = results.as_slice() else {
            return Err(SwapperError::NoQuoteAvailable);
        };

        if hub_pool.decoded_paused_call3(paused)? {
            return Err(SwapperError::ComputeQuoteError("Across protocol is paused".into()));
        }
        if amount > hub_pool.decoded_pooled_token_call3(pooled_token)?.liquidReserves {
            return Err(SwapperError::ComputeQuoteError("Bridge amount is too large".into()));
        }

        let rate_model = TokenConfig::decode(config_response)?.rate_model(input, output)?;
        let utilization_before = hub_pool.decoded_utilization_call3(utilization_before)?;
        let utilization_after = hub_pool.decoded_utilization_call3(utilization_after)?;
        Ok(HubPoolQuote {
            lp_fee_percent: LpFeeCalculator::new(rate_model).realized_lp_fee_pct(&utilization_before, &utilization_after, false),
            timestamp: hub_pool.decoded_current_time(current_time)?,
        })
    }

    async fn usd_price(&self, price: TokenPrice) -> Result<U256, SwapperError> {
        match price {
            TokenPrice::Dollar => Ok(U256::from(ONE_DOLLAR)),
            TokenPrice::Chainlink { chain, feed } => chainlink::latest_price(self.rpc_provider.clone(), chain, feed).await,
        }
    }

    async fn gas_fee(&self, chain: Chain, fill_cost: FillCost, output_price: TokenPrice, decimals: u32) -> Result<U256, SwapperError> {
        let gas_price = async { Self::gas_value(&create_client_with_chain(self.rpc_provider.clone(), chain)?.request::<String, _>(EthereumRpc::GasPrice).await?) };
        let prices = async {
            match fill_cost.gas_token == output_price {
                true => Ok((U256::from(1), U256::from(1))),
                false => try_join!(self.usd_price(fill_cost.gas_token), self.usd_price(output_price)),
            }
        };
        let (gas_price, (gas_token_usd, output_usd)) = try_join!(gas_price, prices)?;
        Ok(Self::gas_fee_in_output(U256::from(fill_cost.gas) * gas_price, gas_token_usd, output_usd, decimals))
    }

    fn gas_fee_in_output(native_fee: U256, gas_token_usd: U256, output_usd: U256, decimals: u32) -> U256 {
        let scale = |decimals: u32| U256::from(10).pow(U256::from(decimals));
        native_fee * gas_token_usd * scale(decimals) / (output_usd * scale(GAS_TOKEN_DECIMALS))
    }

    fn message_for_multicall_handler(amount: &U256, output_funding: Funding, output_token: &Address, user_address: &Address, output_chain: Chain, referral_fee: &ReferralFee) -> Result<(Vec<u8>, U256), SwapperError> {
        if referral_fee.bps == 0 {
            return Ok((vec![], U256::from(0)));
        }
        let fee_address = eth_address::parse(output_chain, &referral_fee.address)?;
        let fee_amount = bps_of(amount, referral_fee.bps);
        let user_amount = amount - fee_amount;

        let calls = match output_funding {
            Funding::Value => Self::unwrap_weth_calls(output_token, amount, user_address, &user_amount, &fee_address, &fee_amount),
            Funding::Token => Self::erc20_transfer_calls(output_token, user_address, &user_amount, &fee_address, &fee_amount),
        };
        let instructions = multicall_handler::Instructions { calls, fallbackRecipient: *user_address };
        let message = instructions.abi_encode();
        Ok((message, fee_amount))
    }

    fn unwrap_weth_calls(weth_contract: &Address, output_amount: &U256, user_address: &Address, user_amount: &U256, fee_address: &Address, fee_amount: &U256) -> Vec<multicall_handler::Call> {
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
        let user_transfer = IERC20::transferCall { to: *user_address, value: *user_amount };
        let fee_transfer = IERC20::transferCall { to: *fee_address, value: *fee_amount };
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

    fn get_eta_in_seconds(&self, from_chain: &Chain, to_chain: &Chain) -> Option<u32> {
        let from_chain = EVMChain::from_chain(*from_chain)?;
        let to_chain = EVMChain::from_chain(*to_chain)?;
        let from_chain_l2 = from_chain.is_ethereum_layer2();
        let to_chain_l2 = to_chain.is_ethereum_layer2();
        Some(match (from_chain_l2, to_chain_l2) {
            (true, true) => 5,
            (true, false) => 10,
            (false, _) => 20,
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
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        if from_chain == to_chain {
            return Err(SwapperError::NoQuoteAvailable);
        }
        if from_chain == Chain::Tron && request.from_asset.is_native() {
            return Err(SwapperError::NotSupportedAsset);
        }
        let depositor_address = eth_address::parse(from_chain, &request.wallet_address)?;
        let recipient_address = eth_address::parse(to_chain, &request.destination_address)?;

        let origin_deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&to_chain).ok_or(SwapperError::NotSupportedChain)?;
        let fill_cost = AcrossDeployment::fill_cost(&to_chain).ok_or(SwapperError::NotSupportedChain)?;
        if !Self::is_supported_route(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let input = AcrossAsset::from_asset(&request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let output = AcrossAsset::from_asset(&request.to_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let from_amount = U256::from_str(&request.value.to_string()).map_err(SwapperError::from)? / input.scale;
        let input_token = eth_address::parse(input.asset_id.chain, input.asset_id.token_id.as_deref().ok_or(SwapperError::NotSupportedAsset)?)?;
        let output_token = eth_address::parse(output.asset_id.chain, output.asset_id.token_id.as_deref().ok_or(SwapperError::NotSupportedAsset)?)?;

        let mappings = AcrossDeployment::asset_mappings();
        let mapping = |asset: &AssetId| mappings.iter().find(|x| x.set.contains(asset)).ok_or(SwapperError::NotSupportedAsset);
        let asset_mapping = mapping(&input.asset_id)?;
        let output_price = asset_mapping.price.ok_or(SwapperError::NotSupportedAsset)?;
        let asset_mainnet = asset_mapping.set.iter().find(|x| x.chain == Chain::Ethereum).ok_or(SwapperError::NotSupportedAsset)?;
        let mainnet_token = eth_address::parse_asset_id(asset_mainnet)?;
        let cost_config = &asset_mapping.capital_cost;
        let output_decimals = mapping(&output.asset_id)?.capital_cost.decimals;
        let rescale = |decimals| biguint_to_u256(&rescale_base_units(u256_to_biguint(&from_amount), cost_config.decimals, decimals)).ok_or(SwapperError::InputAmountError { min_amount: None });
        let mainnet_amount = rescale(mapping(asset_mainnet)?.capital_cost.decimals)?;
        let output_amount = rescale(output_decimals)?;

        let (hub_pool, gas_fee) = try_join!(
            self.hub_pool_quote(&mainnet_token, mainnet_amount, &input.asset_id, &output.asset_id),
            self.gas_fee(to_chain, fill_cost, output_price, output_decimals),
        )?;

        let lp_fee = fees::multiply(output_amount, hub_pool.lp_fee_percent, output_decimals);
        let relayer_fee_percent = RelayerFeeCalculator::default().capital_fee_percent(&BigInt::from_str(&from_amount.to_string())?, cost_config);
        let relayer_fee = fees::multiply(output_amount, relayer_fee_percent, output_decimals);
        let output_amount = output_amount
            .checked_sub(lp_fee + relayer_fee + gas_fee)
            .filter(|amount| !amount.is_zero())
            .ok_or(SwapperError::InputAmountError { min_amount: None })?;

        let referral_config = referral_fees(BRIDGE_FEE_BPS).for_chain(to_chain).cloned().unwrap_or_default();
        let (message, referral_fee) = Self::message_for_multicall_handler(&output_amount, output.funding, &output_token, &recipient_address, to_chain, &referral_config)?;
        let recipient = if message.is_empty() {
            recipient_address
        } else {
            eth_address::parse(to_chain, destination_deployment.multicall_handler())?
        };
        let relay_data = V3RelayData {
            depositor: depositor_address,
            recipient,
            exclusiveRelayer: Address::ZERO,
            inputToken: input_token,
            outputToken: output_token,
            inputAmount: from_amount,
            outputAmount: output_amount,
            originChainId: U256::from(origin_deployment.chain_id),
            depositId: u32::MAX,
            fillDeadline: hub_pool.timestamp + DEFAULT_FILL_TIMEOUT,
            exclusivityDeadline: 0,
            message: message.into(),
        };
        let to_value = (output_amount - referral_fee) * output.scale;

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: u256_to_biguint(&to_value),
            data: ProviderData {
                provider: self.provider().clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: input.asset_id,
                    output: output.asset_id,
                    route_data: HexEncode(relay_data.abi_encode()),
                }],
            },
            request: request.clone(),
            eta_in_seconds: self.get_eta_in_seconds(&from_chain, &to_chain),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_chain = quote.request.from_asset.chain();
        let deployment = AcrossDeployment::deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let destination_deployment = AcrossDeployment::deployment_by_chain(&quote.request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data = HexDecode(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let v3_relay_data = V3RelayData::abi_decode(&route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let quote_timestamp = v3_relay_data.fillDeadline.checked_sub(DEFAULT_FILL_TIMEOUT).ok_or(SwapperError::InvalidRoute)?;

        let deposit_v3_call = V3SpokePoolInterface::depositV3Call {
            depositor: v3_relay_data.depositor,
            recipient: v3_relay_data.recipient,
            inputToken: v3_relay_data.inputToken,
            outputToken: v3_relay_data.outputToken,
            inputAmount: v3_relay_data.inputAmount,
            outputAmount: v3_relay_data.outputAmount,
            destinationChainId: U256::from(destination_deployment.chain_id),
            exclusiveRelayer: Address::ZERO,
            quoteTimestamp: quote_timestamp,
            fillDeadline: v3_relay_data.fillDeadline,
            exclusivityDeadline: 0,
            message: v3_relay_data.message,
        }
        .abi_encode();

        let input = AcrossAsset::from_asset(&quote.request.from_asset.asset_id()).ok_or(SwapperError::NotSupportedAsset)?;
        let value = match input.funding {
            Funding::Value => quote.from_value.clone(),
            Funding::Token => BigUint::ZERO,
        };

        let approval = match input.funding {
            Funding::Value => None,
            Funding::Token => {
                let token = match from_chain {
                    Chain::Tron => quote.request.from_asset.asset_id().token_id.ok_or(SwapperError::NotSupportedAsset)?,
                    _ => v3_relay_data.inputToken.to_string(),
                };
                check_token_approval(quote.request.wallet_address.clone(), token, deployment.spoke_pool.into(), v3_relay_data.inputAmount, self.rpc_provider.clone(), from_chain).await?
            }
        };

        let gas_limit = match (from_chain, data) {
            (Chain::Tron, _) => None,
            (_, FetchQuoteData::EstimateGas) => {
                let tx = TransactionObject::new_call_to_value(deployment.spoke_pool, &format!("{value:#x}"), deposit_v3_call.clone());
                Some(self.estimate_gas_transaction(from_chain, tx).await?.to_string())
            }
            (_, FetchQuoteData::Permit2(_) | FetchQuoteData::None) => get_swap_gas_limit_with_approval(&approval, None, DEFAULT_DEPOSIT_GAS_LIMIT),
        };

        Ok(SwapperQuoteData::new_contract(deployment.spoke_pool.into(), value, HexEncode(deposit_v3_call), approval, gas_limit))
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses {
            deposit: AcrossDeployment::deposit_addresses(),
            send: AcrossDeployment::send_addresses(),
        })
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        super::status::get_swap_result(self.rpc_provider.clone(), request.chain, &request.transaction_hash).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{TEST_FILL_DEADLINE, mock_v3_relay_data};
    use super::*;
    use crate::alien::mock::ProviderMock;
    use num_bigint::BigUint;
    use primitives::{
        asset_constants::*,
        contract_constants::ARC_ACROSS_SPOKE_POOL_CONTRACT,
        swap::{ApprovalData, SwapQuoteDataType},
    };

    #[test]
    fn test_is_supported_route() {
        let weth_eth: AssetId = ETHEREUM_WETH_ASSET_ID.clone();
        let weth_op: AssetId = OPTIMISM_WETH_ASSET_ID.clone();
        let weth_arb: AssetId = ARBITRUM_WETH_ASSET_ID.clone();
        let weth_blast: AssetId = BLAST_WETH_ASSET_ID.clone();
        let weth_bsc: AssetId = SMARTCHAIN_ETH_ASSET_ID.clone();
        let weth_robinhood: AssetId = ROBINHOOD_WETH_ASSET_ID.clone();

        let usdc_eth: AssetId = ETHEREUM_USDC_ASSET_ID.clone();
        let usdc_arb: AssetId = ARBITRUM_USDC_ASSET_ID.clone();
        let usdc_avalanche: AssetId = AVALANCHE_USDC_ASSET_ID.clone();
        let usdc_monad: AssetId = MONAD_USDC_ASSET_ID.clone();
        let usdg_robinhood: AssetId = ROBINHOOD_USDG_ASSET_ID.clone();
        let usdt_eth: AssetId = ETHEREUM_USDT_ASSET_ID.clone();
        let usdt_avalanche: AssetId = AVALANCHE_USDT_ASSET_ID.clone();
        let usdt_monad: AssetId = MONAD_USDT_ASSET_ID.clone();
        let usdt_tron: AssetId = TRON_USDT_ASSET_ID.clone();

        assert!(Across::is_supported_route(&weth_eth, &weth_op));
        assert!(Across::is_supported_route(&weth_op, &weth_arb));
        assert!(!Across::is_supported_route(&weth_eth, &weth_blast));
        assert!(!Across::is_supported_route(&weth_blast, &weth_eth));
        assert!(Across::is_supported_route(&usdc_eth, &usdc_arb));
        assert!(Across::is_supported_route(&usdc_eth, &usdc_avalanche));
        assert!(Across::is_supported_route(&usdt_avalanche, &usdt_eth));
        assert!(Across::is_supported_route(&usdc_monad, &usdc_eth));
        assert!(Across::is_supported_route(&usdt_monad, &usdt_eth));
        assert!(Across::is_supported_route(&usdt_tron, &usdt_eth));
        assert!(!Across::is_supported_route(&usdt_eth, &usdt_tron));
        assert!(Across::is_supported_route(&weth_eth, &weth_bsc));
        assert!(Across::is_supported_route(&weth_robinhood, &weth_eth));
        assert!(Across::is_supported_route(&weth_eth, &weth_robinhood));
        assert!(Across::is_supported_route(&usdc_eth, &usdg_robinhood));
        assert!(Across::is_supported_route(&usdg_robinhood, &usdc_eth));

        assert!(!Across::is_supported_route(&weth_eth, &usdc_eth));
        assert!(!Across::is_supported_route(&weth_eth, &usdt_tron));

        let eth = AssetId::from(Chain::Ethereum, None);
        let op = AssetId::from(Chain::Optimism, None);
        let arb = AssetId::from(Chain::Arbitrum, None);
        let linea = AssetId::from(Chain::Linea, None);

        assert!(Across::is_supported_route(&eth, &linea));
        assert!(Across::is_supported_route(&op, &eth));
        assert!(Across::is_supported_route(&arb, &eth));
        assert!(Across::is_supported_route(&op, &arb));
        assert!(Across::is_supported_route(&eth, &AssetId::from(Chain::Robinhood, None)));
        assert!(Across::is_supported_route(&AssetId::from(Chain::Robinhood, None), &eth));

        let arc = AssetId::from_chain(Chain::Arc);
        assert!(Across::is_supported_route(&arc, &usdc_eth));
        assert!(!Across::is_supported_route(&arc, &eth));

        assert!(Across::is_supported_route(&usdc_eth, &SMARTCHAIN_USDC_ASSET_ID));
        assert!(Across::is_supported_route(&SMARTCHAIN_USDT_ASSET_ID, &usdt_eth));
        assert!(!Across::is_supported_route(&usdc_eth, &ARBITRUM_USDC_E_ASSET_ID));
        assert!(!Across::is_supported_route(&ETHEREUM_WBTC_ASSET_ID, &ARBITRUM_WBTC_ASSET_ID));
    }

    #[tokio::test]
    async fn test_direct_quote_data_maps_tron_approval() {
        let provider = Across::new(Arc::new(ProviderMock::mock_tron_constant_result("0")));
        let relay_data = mock_v3_relay_data();
        let quote = Quote::mock_across_tron();
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
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert_eq!(quote_data.data, HexEncode(expected_data));
        assert_eq!(quote_data.gas_limit, None);
        assert_eq!(
            quote_data.approval,
            Some(ApprovalData {
                token: TRON_USDT_TOKEN_ID.to_string(),
                spender: "TTbCVPfUZmPhrB9sYC8GKgGBQQEdZovkmS".to_string(),
                value: BigUint::from(10000000u64),
                is_unlimited: true,
            })
        );
    }

    #[tokio::test]
    async fn test_direct_quote_data_skips_tron_approval_when_allowance_covers_amount() {
        let provider = Across::new(Arc::new(ProviderMock::mock_tron_constant_result("989680")));
        let quote = Quote::mock_across_tron();
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.approval, None);
        assert_eq!(quote_data.gas_limit, None);
    }

    #[tokio::test]
    async fn test_direct_quote_data_approves_the_token_behind_native_arc_usdc() {
        let provider = Across::new(Arc::new(ProviderMock::mock_json_rpc_result(&HexEncode([0u8; 32]))));
        let quote_data = provider.get_quote_data(&Quote::mock_across_arc(), FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.to, ARC_ACROSS_SPOKE_POOL_CONTRACT);
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert_eq!(
            quote_data.approval,
            Some(ApprovalData {
                token: ARC_USDC_TOKEN_ID.to_string(),
                spender: ARC_ACROSS_SPOKE_POOL_CONTRACT.to_string(),
                value: BigUint::from(5_000_000u64),
                is_unlimited: true,
            })
        );
    }

    #[test]
    fn test_gas_fee_in_output() {
        let gwei = |value: u64| U256::from(value) * U256::from(1_000_000_000u64);
        let usd = |cents: u64| U256::from(cents) * U256::from(1_000_000u64);
        let one = U256::from(1);

        assert_eq!(Across::gas_fee_in_output(U256::from(370_000) * gwei(1) / U256::from(50), usd(240_869), usd(100), 6), U256::from(17_824));
        assert_eq!(Across::gas_fee_in_output(U256::from(190_000) * gwei(1) / U256::from(20), usd(71_953), usd(240_869), 18), U256::from(2_837_864_150_222u64));
        assert_eq!(Across::gas_fee_in_output(U256::from(340_000) * gwei(5), usd(978), usd(100), 6), U256::from(16_626));
        assert_eq!(Across::gas_fee_in_output(U256::from(370_000) * gwei(1) / U256::from(50), one, one, 18), U256::from(7_400_000_000_000u64));
        assert_eq!(Across::gas_fee_in_output(U256::from(310_000) * gwei(21), one, one, 6), U256::from(6_510));
    }

    #[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
    mod swap_integration_tests {
        use super::*;
        use crate::{FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError};
        use primitives::{AssetId, Chain, swap::SwapStatus};
        use std::{sync::Arc, time::SystemTime};

        #[tokio::test]
        async fn test_across_quotes_charge_relayer_costs_in_the_output_token() -> Result<(), SwapperError> {
            let usdc = 50_000_000u128;
            let bsc_usdc = 50_000_000_000_000_000_000u128;
            let ether = 20_000_000_000_000_000u128;
            let routes = [
                (ETHEREUM_USDC_ASSET_ID.clone(), SMARTCHAIN_USDC_ASSET_ID.clone(), usdc),
                (SMARTCHAIN_USDC_ASSET_ID.clone(), ETHEREUM_USDC_ASSET_ID.clone(), bsc_usdc),
                (ETHEREUM_USDT_ASSET_ID.clone(), SMARTCHAIN_USDT_ASSET_ID.clone(), usdc),
                (SMARTCHAIN_USDT_ASSET_ID.clone(), ETHEREUM_USDT_ASSET_ID.clone(), bsc_usdc),
                (ETHEREUM_USDC_ASSET_ID.clone(), AVALANCHE_USDC_ASSET_ID.clone(), usdc),
                (ETHEREUM_USDC_ASSET_ID.clone(), POLYGON_USDC_ASSET_ID.clone(), usdc),
                (ARBITRUM_USDC_ASSET_ID.clone(), HYPEREVM_USDC_ASSET_ID.clone(), usdc),
                (ETHEREUM_USDC_ASSET_ID.clone(), MONAD_USDC_ASSET_ID.clone(), usdc),
                (ETHEREUM_USDT_ASSET_ID.clone(), PLASMA_USDT_ASSET_ID.clone(), usdc),
                (ETHEREUM_USDC_ASSET_ID.clone(), ARBITRUM_USDC_ASSET_ID.clone(), usdc),
                (ETHEREUM_USDC_ASSET_ID.clone(), AssetId::from_chain(Chain::Arc), usdc),
                (ETHEREUM_WETH_ASSET_ID.clone(), SMARTCHAIN_ETH_ASSET_ID.clone(), ether),
                (ETHEREUM_WETH_ASSET_ID.clone(), POLYGON_WETH_ASSET_ID.clone(), ether),
                (OPTIMISM_WETH_ASSET_ID.clone(), ARBITRUM_WETH_ASSET_ID.clone(), ether),
                (AssetId::from_chain(Chain::Arbitrum), AssetId::from_chain(Chain::Base), ether),
                (AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Linea), ether),
                (AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Robinhood), ether),
            ];
            let chains = routes.iter().flat_map(|(from, to, _)| [from.chain, to.chain]).chain([Chain::Ethereum]);
            let across = Across::new(crate::testkit::dynode_provider(chains));
            let decimals = |asset: &AssetId| {
                let asset_id = AcrossAsset::from_asset(asset).unwrap().asset_id;
                AcrossDeployment::asset_mappings().into_iter().find(|x| x.set.contains(&asset_id)).unwrap().capital_cost.decimals
            };

            for (from_asset, to_asset, value) in routes {
                let output_value = rescale_base_units(BigUint::from(value), decimals(&from_asset), decimals(&to_asset));
                let request = QuoteRequest {
                    value: BigUint::from(value),
                    ..crate::testkit::mock_quote(from_asset.clone().into(), to_asset.clone().into())
                };
                let quote = across.get_quote(&request).await?;
                let relay_data = V3RelayData::abi_decode(&HexDecode(&quote.data.routes[0].route_data).unwrap()).unwrap();
                let scale = AcrossAsset::from_asset(&to_asset).unwrap().scale;
                println!("{from_asset} -> {to_asset}: output {} to_value {}", relay_data.outputAmount, quote.to_value);

                assert!(quote.to_value * BigUint::from(1000u32) >= &output_value * u256_to_biguint(&scale) * BigUint::from(990u32), "{from_asset} -> {to_asset}");
                assert!(u256_to_biguint(&relay_data.outputAmount) < output_value, "{from_asset} -> {to_asset}");
            }

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options { slippage: 100.into(), use_max_amount: false };

            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Optimism).into(),
                to_asset: AssetId::from_chain(Chain::Arbitrum).into(),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                value: BigUint::from(20000000000000000u64),
                options,
            };

            let now = SystemTime::now();
            let quote = swap_provider.get_quote(&request).await?;
            let elapsed = SystemTime::now().duration_since(now).unwrap();

            println!("<== elapsed: {:?}", elapsed);
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value > BigUint::ZERO);

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
            println!("<== quote_data: {:?}", quote_data);

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote_eth_usdc_to_monad_usdc() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider.clone());
            let options = Options { slippage: 100.into(), use_max_amount: false };

            let wallet = "0x9b1fe00135e0ff09389bfaeff0c8f299ec818d4a";
            let from_asset: AssetId = ETHEREUM_USDC_ASSET_ID.clone();
            let to_asset: AssetId = MONAD_USDC_ASSET_ID.clone();
            let request = QuoteRequest {
                from_asset: from_asset.into(),
                to_asset: to_asset.into(),
                wallet_address: wallet.into(),
                destination_address: wallet.into(),
                value: BigUint::from(50000000u64),
                options,
            };

            let now = SystemTime::now();
            let quote = swap_provider.get_quote(&request).await?;
            let elapsed = SystemTime::now().duration_since(now).unwrap();

            println!("<== elapsed: {:?}", elapsed);
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value > BigUint::ZERO);

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            println!("<== quote_data: {:?}", quote_data);

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote_ethereum_usdc_to_avalanche_usdc() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider);
            let wallet = "0x9b1fe00135e0ff09389bfaeff0c8f299ec818d4a";
            let request = QuoteRequest {
                from_asset: ETHEREUM_USDC_ASSET_ID.clone().into(),
                to_asset: AVALANCHE_USDC_ASSET_ID.clone().into(),
                wallet_address: wallet.into(),
                destination_address: wallet.into(),
                value: BigUint::from(50000000u64),
                options: Options { slippage: 100.into(), use_max_amount: false },
            };

            let quote = swap_provider.get_quote(&request).await?;
            assert!(quote.to_value > BigUint::ZERO);
            assert!(!quote.data.routes[0].route_data.is_empty());

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote_eth_to_robinhood() -> Result<(), SwapperError> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::boxed(network_provider);
            let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Ethereum).into(),
                to_asset: AssetId::from_chain(Chain::Robinhood).into(),
                wallet_address: wallet.into(),
                destination_address: wallet.into(),
                value: BigUint::from(20000000000000000u64),
                options: Options { slippage: 100.into(), use_max_amount: false },
            };

            let quote = swap_provider.get_quote(&request).await?;
            assert!(quote.to_value > BigUint::ZERO);
            assert!(!quote.data.routes[0].route_data.is_empty());

            Ok(())
        }

        #[tokio::test]
        async fn test_across_quote_arc_usdc_to_base_usdc() -> Result<(), SwapperError> {
            let swap_provider = Across::boxed(Arc::new(NativeProvider::default()));
            let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
            let request = QuoteRequest {
                from_asset: AssetId::from_chain(Chain::Arc).into(),
                to_asset: BASE_USDC_ASSET_ID.clone().into(),
                wallet_address: wallet.into(),
                destination_address: wallet.into(),
                value: BigUint::from(5_000_000_000_000_000_000u64),
                options: Options::default(),
            };

            let quote = swap_provider.get_quote(&request).await?;
            println!("<== quote: {:?}", quote);
            assert!(quote.to_value > BigUint::from(4_900_000u64) && quote.to_value < BigUint::from(5_000_000u64));

            let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            println!("<== quote_data: {:?}", quote_data);
            assert_eq!(quote_data.value, BigUint::ZERO);

            Ok(())
        }

        #[tokio::test]
        async fn test_get_swap_result_arc_usdc_to_ethereum_usdc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let swap_provider = Across::new(crate::testkit::dynode_provider([Chain::Arc]));
            let tx_hash = "0x026d408d6f548824ccec8dd2fa1381a5b176a2dc357058ee501a062f9008b703";

            let result = swap_provider.get_swap_result(&SwapResultRequest::new(Chain::Arc, tx_hash)).await?;
            let metadata = result.metadata.unwrap();

            assert_eq!(result.status, SwapStatus::Completed);
            assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Arc));
            assert_eq!(metadata.from_value, BigUint::from(5_000_000_000_000_000_000u64));
            assert_eq!(metadata.to_asset, ETHEREUM_USDC_ASSET_ID.clone());
            assert_eq!(metadata.to_value, BigUint::from(4_984_358u64));

            Ok(())
        }

        #[tokio::test]
        async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let network_provider = Arc::new(NativeProvider::default());
            let swap_provider = Across::new(network_provider.clone());

            let tx_hash = "0x0a970040a9885cf2c8a42df6fcdf02a1f3fe7db12079a35613a665a2ee64df49";
            let chain = Chain::Arbitrum;

            let result = swap_provider.get_swap_result(&SwapResultRequest::new(chain, tx_hash)).await?;

            println!("Across swap result: {:?}", result);
            assert_eq!(result.status, SwapStatus::Completed);

            let metadata = result.metadata.unwrap();
            assert_eq!(metadata.provider, Some("across".to_string()));
            assert!(metadata.from_value > BigUint::ZERO);
            assert!(metadata.to_value > BigUint::ZERO);

            Ok(())
        }
    }
}
