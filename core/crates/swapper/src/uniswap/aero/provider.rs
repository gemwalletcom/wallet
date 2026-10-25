use super::commands::{SwapCommands, build_commands, encode_execute, input_fee};
use crate::{
    Contracts, FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::RpcProvider,
    approval::{DEFAULT_EVM_SWAP_GAS_LIMIT, check_approval_erc20, get_swap_gas_limit_with_approval},
    client_factory::create_client_with_chain,
    eth_address,
    fees::on_chain_referral_fees,
    uniswap::{
        deadline::get_sig_deadline,
        discovery::{PoolDiscovery, candidate_pairs, discover_pools, v3_pool_request},
        fee_token::is_quote_input_fee_token,
        quote_result::{OutpricedPaths, get_best_quote},
        routed_asset::{Funding, Protocol, base_pair, routed_pair, routed_request},
        swap_route::{PoolPath, RouteData, candidate_paths, get_intermediaries},
    },
};
use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_evm::{
    jsonrpc::EthereumRpc,
    u256::u256_to_biguint,
    uniswap::{
        FeeTier,
        contracts::aero::IMetaquoter,
        deployment::{
            aero::{AeroDeployment, get_aero_deployment_by_chain},
            v3::PoolLookup,
        },
        path::TokenPair,
    },
};
use gem_jsonrpc::types::JsonRpcResponse;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, decode_hex};
use std::{fmt, str::FromStr, sync::Arc};

const PROTOCOL: Protocol = Protocol::V3;
const TICK_SPACINGS: [FeeTier; 5] = [FeeTier::One, FeeTier::Ten, FeeTier::Fifty, FeeTier::Hundred, FeeTier::TwoHundred];
const SWAP_GAS_LIMIT: u64 = 300_000;
const POOL_GAS_LIMIT: u64 = 450_000;

pub struct Aero {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    pool_discovery: PoolDiscovery,
    outpriced_paths: OutpricedPaths,
}

impl Aero {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Aero),
            rpc_provider,
            pool_discovery: PoolDiscovery::default(),
            outpriced_paths: OutpricedPaths::default(),
        }
    }

    fn deployment(chain: Chain) -> Result<AeroDeployment, SwapperError> {
        get_aero_deployment_by_chain(&chain).ok_or(SwapperError::NotSupportedChain)
    }

    async fn preload_pools(&self, from_asset: &AssetId, to_asset: &AssetId) -> Result<(), SwapperError> {
        let chain = from_asset.chain;
        let deployment = Self::deployment(chain)?;
        let (evm_chain, input, output) = routed_pair(from_asset, to_asset, PROTOCOL)?;
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or(SwapperError::NotSupportedChain)?;
        let pairs = candidate_pairs(input.address, output.address, get_intermediaries(&input.address, &output.address, &base_pair));
        let factory = deployment.factory.to_checksum(None);
        let pools = self
            .pool_discovery
            .missing_pools(chain, &pairs, &TICK_SPACINGS)
            .into_iter()
            .map(|pool| {
                let request = v3_pool_request(&factory, PoolLookup::TickSpacing, &pool);
                (pool, request)
            })
            .collect::<Vec<_>>();
        if pools.is_empty() {
            return Ok(());
        }
        let client = create_client_with_chain(self.rpc_provider.clone(), chain)?;
        let discovered = discover_pools(&client, pools).await?;
        self.pool_discovery.record_pools(chain, &discovered);
        Ok(())
    }
}

fn quote_request(deployment: &AeroDeployment, token_in: Address, pairs: &[TokenPair], amount_in: U256) -> EthereumRpc {
    let call = IMetaquoter::quoteExactInputCall {
        pools: pairs.iter().map(|pair| deployment.pool_address(pair)).collect(),
        tokenIn: token_in,
        amountIn: amount_in,
    };
    EthereumRpc::call(deployment.metaquoter, call.abi_encode())
}

fn decode_quote_response(response: &JsonRpcResponse<String>) -> Result<U256, SwapperError> {
    let bytes = decode_hex(&response.result).map_err(SwapperError::compute_quote_error)?;
    let quote = IMetaquoter::quoteExactInputCall::abi_decode_returns(&bytes)?;
    Some(quote.amountOut).filter(|amount_out| !amount_out.is_zero()).ok_or(SwapperError::NoQuoteAvailable)
}

fn route_pools(deployment: &AeroDeployment, quote: &Quote, token_in: Address, token_out: Address) -> Result<Vec<Address>, SwapperError> {
    let pairs = quote.data.routes.iter().map(TokenPair::try_from).collect::<Result<Vec<_>, _>>()?;
    let connected = pairs.first().map(|pair| pair.token_in) == Some(token_in) && pairs.last().map(|pair| pair.token_out) == Some(token_out) && pairs.windows(2).all(|hops| hops[0].token_out == hops[1].token_in);
    if !connected {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(pairs.iter().map(|pair| deployment.pool_address(pair)).collect())
}

impl fmt::Debug for Aero {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Aero").finish()
    }
}

#[async_trait]
impl Swapper for Aero {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        Chain::all().into_iter().filter(|chain| get_aero_deployment_by_chain(chain).is_some()).map(SwapperChainAsset::All).collect()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    fn contracts(&self, from_asset: &AssetId) -> Contracts {
        get_aero_deployment_by_chain(&from_asset.chain).map(|deployment| Contracts::new(vec![deployment.metarouter])).unwrap_or_default()
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        _ = self.preload_pools(from_asset, to_asset).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let chain = request.from_asset.chain();
        let deployment = Self::deployment(chain)?;
        let (evm_chain, input, output, from_value) = routed_request(request, PROTOCOL)?;
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or(SwapperError::NotSupportedChain)?;
        let fee_token_is_input = is_quote_input_fee_token(Some(&base_pair), request, input.address, output.address);
        let amount_in = from_value - input_fee(from_value, on_chain_referral_fees(request).evm.bps, fee_token_is_input);

        _ = self.preload_pools(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;
        let paths = candidate_paths(&input.address, &output.address, &TICK_SPACINGS, &base_pair)
            .into_iter()
            .filter(|pairs| self.pool_discovery.path_may_exist(chain, pairs))
            .map(|pairs| {
                let request = quote_request(&deployment, input.address, &pairs, amount_in);
                (PoolPath::new(PROTOCOL, pairs), request)
            })
            .collect();
        let client = create_client_with_chain(self.rpc_provider.clone(), chain)?;
        let quote = get_best_quote(&client, &self.outpriced_paths, (chain, input.address), (chain, output.address), amount_in.saturating_to(), paths, decode_quote_response).await?;

        Ok(quote.into_quote(request, self.provider.clone(), fee_token_is_input, output.scale))
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let chain = request.from_asset.chain();
        let deployment = Self::deployment(chain)?;
        let (evm_chain, input, output, amount_in) = routed_request(request, PROTOCOL)?;
        let route_data: RouteData = quote.route_data()?;
        let pools = route_pools(&deployment, quote, input.address, output.address)?;

        let approval = match input.funding {
            Funding::Permit2 => check_approval_erc20(request.wallet_address.clone(), input.address.to_checksum(None), deployment.metarouter.to_string(), amount_in, self.rpc_provider.clone(), &chain).await?,
            Funding::Value | Funding::RouterBalance => None,
        };
        let swap_gas_limit = SWAP_GAS_LIMIT + POOL_GAS_LIMIT * pools.len() as u64;
        let gas_limit = get_swap_gas_limit_with_approval(&approval, Some(swap_gas_limit.to_string()), DEFAULT_EVM_SWAP_GAS_LIMIT);
        let value = match input.funding {
            Funding::Value | Funding::RouterBalance => u256_to_biguint(&(amount_in * input.scale)),
            Funding::Permit2 => BigUint::ZERO,
        };

        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or(SwapperError::NotSupportedChain)?;
        let commands = build_commands(&SwapCommands {
            fee_token_is_input: is_quote_input_fee_token(Some(&base_pair), request, input.address, output.address),
            input,
            output,
            pools,
            amount_in,
            min_amount_out: U256::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?,
            recipient: eth_address::parse_str(&request.wallet_address)?,
            metarouter: eth_address::parse_str(deployment.metarouter)?,
            referral: on_chain_referral_fees(request).evm,
        })?;
        let calldata = encode_execute(commands, U256::from(get_sig_deadline()));

        Ok(SwapperQuoteData::new_contract(deployment.metarouter.to_string(), value, HexEncode(calldata), approval, gas_limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        alien::mock::ProviderMock,
        fees::subtract_bps,
        testkit::mock_quote,
        uniswap::{quote_result::QuoteResult, testkit::mock_batch_provider},
    };
    use alloy_primitives::Bytes;
    use alloy_sol_types::SolValue;
    use gem_evm::uniswap::contracts::{
        aero::{BALANCE_CHECK_COMMAND, BalanceCheck, BalanceSpend, CL_SWAP_EXACT_IN_COMMAND, IMetarouter, SwapExactIn, TRANSFER_COMMAND, Transfer, UNWRAP_WETH_COMMAND, UnwrapWeth, WRAP_ETH_COMMAND},
        v3::ISlipstreamFactory,
    };
    use primitives::{
        asset_constants::{BASE_CBBTC_ASSET_ID, BASE_CBBTC_TOKEN_ID, BASE_USDC_ASSET_ID, BASE_USDC_TOKEN_ID, BASE_USDT_TOKEN_ID, BASE_WETH_TOKEN_ID},
        hex::encode_with_0x,
        swap::EVM_REFERRAL_ADDRESS,
    };
    use serde_json::json;
    use std::mem::take;

    fn decode_execute(data: &SwapperQuoteData) -> (Vec<u8>, Vec<Bytes>) {
        let execute = IMetarouter::executeCall::abi_decode(&decode_hex(&data.data).unwrap()).unwrap();
        (execute.commands.to_vec(), execute.inputs)
    }

    #[tokio::test]
    async fn test_get_quote() {
        let (weth, usdc): (Address, Address) = (BASE_WETH_TOKEN_ID.parse().unwrap(), BASE_USDC_TOKEN_ID.parse().unwrap());
        let deployment = get_aero_deployment_by_chain(&Chain::Base).unwrap();
        let pair = TokenPair {
            token_in: weth,
            token_out: usdc,
            fee_tier: FeeTier::Fifty,
        };
        let pool = deployment.pool_address(&pair);
        let request = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            ..mock_quote(AssetId::from_chain(Chain::Base).into(), BASE_USDC_ASSET_ID.clone().into())
        };
        let (provider, posts) = mock_batch_provider(move |rpc| {
            let data = decode_hex(rpc["params"][0]["data"].as_str().unwrap()).unwrap();
            let result = match ISlipstreamFactory::getPoolCall::abi_decode(&data) {
                Ok(call) => {
                    let exists = [call.tokenA, call.tokenB] == [weth, usdc] && call.tickSpacing == FeeTier::Fifty.as_tick_spacing();
                    encode_with_0x(&ISlipstreamFactory::getPoolCall::abi_encode_returns(&if exists { pool } else { Address::ZERO }))
                }
                Err(_) => {
                    let call = IMetaquoter::quoteExactInputCall::abi_decode(&data).unwrap();
                    assert_eq!((call.pools, call.tokenIn, call.amountIn), (vec![pool], weth, U256::from(995_000_000_000_000_000u64)));
                    encode_with_0x(&IMetaquoter::quoteExactInputCall::abi_encode_returns(&IMetaquoter::quoteExactInputReturn {
                        amountOut: U256::from(2_380_000_000u64),
                        sqrtPriceX96AfterList: vec![],
                        initializedTicksCrossedList: vec![],
                    }))
                }
            };
            json!({"jsonrpc": "2.0", "id": rpc["id"], "result": result})
        });
        let aero = Aero::new(Arc::new(provider));

        let quote = aero.get_quote(&request).await.unwrap();
        assert_eq!(take(&mut *posts.lock().unwrap()), vec![15, 1]);
        assert_eq!(aero.get_quote(&request).await.unwrap().to_value, quote.to_value);
        assert_eq!(take(&mut *posts.lock().unwrap()), vec![1]);

        let route_data: RouteData = quote.route_data().unwrap();
        assert_eq!(quote.to_value, BigUint::from(2_380_000_000u64));
        assert_eq!((route_data.fee_tier.as_str(), route_data.min_amount_out.as_str(), quote.data.routes.len()), ("50", "2368100000", 1));
    }

    #[tokio::test]
    async fn test_get_quote_data() {
        let (weth, usdc): (Address, Address) = (BASE_WETH_TOKEN_ID.parse().unwrap(), BASE_USDC_TOKEN_ID.parse().unwrap());
        let deployment = get_aero_deployment_by_chain(&Chain::Base).unwrap();
        let metarouter: Address = deployment.metarouter.parse().unwrap();
        let referral: Address = EVM_REFERRAL_ADDRESS.parse().unwrap();
        let weth_to_usdc = TokenPair {
            token_in: weth,
            token_out: usdc,
            fee_tier: FeeTier::Fifty,
        };
        let usdc_to_weth = TokenPair {
            token_in: usdc,
            token_out: weth,
            fee_tier: FeeTier::Fifty,
        };
        let pool = deployment.pool_address(&weth_to_usdc);
        let native = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            ..mock_quote(AssetId::from_chain(Chain::Base).into(), BASE_USDC_ASSET_ID.clone().into())
        };
        let token = QuoteRequest {
            from_asset: native.to_asset.clone(),
            to_asset: native.from_asset.clone(),
            value: BigUint::from(100_000_000u64),
            ..native.clone()
        };
        let allowance = ProviderMock::mock_json_rpc_result(&encode_with_0x(&U256::ZERO.abi_encode()));
        let aero = Aero::new(Arc::new(allowance));

        let quote = QuoteResult {
            amount_out: U256::from(2_380_000_000u64),
            path: PoolPath::new(PROTOCOL, vec![weth_to_usdc]),
        }
        .into_quote(&native, aero.provider().clone(), true, U256::from(1));
        let data = aero.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let (commands, inputs) = decode_execute(&data);
        (&aero as &dyn Swapper).verify_contracts(&quote, &data).unwrap();

        assert_eq!((data.to.as_str(), data.value.clone(), data.approval.clone(), data.gas_limit.clone()), (deployment.metarouter, native.value.clone(), None, None));
        assert_eq!(commands, vec![WRAP_ETH_COMMAND, TRANSFER_COMMAND, CL_SWAP_EXACT_IN_COMMAND]);
        assert_eq!(BalanceSpend::abi_decode(&inputs[0]).unwrap(), BalanceSpend::amount(U256::from(1_000_000_000_000_000_000u64)));
        assert_eq!(
            Transfer::abi_decode(&inputs[1]).unwrap(),
            Transfer {
                token: weth,
                recipient: referral,
                spend: BalanceSpend::amount(U256::from(5_000_000_000_000_000u64))
            }
        );
        assert_eq!(
            SwapExactIn::abi_decode(&inputs[2]).unwrap(),
            SwapExactIn {
                pools: vec![pool],
                tokenIn: weth,
                amountIn: BalanceSpend::amount(U256::from(995_000_000_000_000_000u64)),
                payerIsUser: false,
                minAmountOut: U256::from(2_368_100_000u64),
                recipient: native.wallet_address.parse().unwrap(),
            }
        );

        let amount_out = U256::from(420_000_000_000_000_000u64);
        let min_amount_out = subtract_bps(&subtract_bps(&amount_out, 50), token.options.slippage.bps);
        let quote = QuoteResult {
            amount_out,
            path: PoolPath::new(PROTOCOL, vec![usdc_to_weth]),
        }
        .into_quote(&token, aero.provider().clone(), false, U256::from(1));
        let data = aero.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let (commands, inputs) = decode_execute(&data);
        (&aero as &dyn Swapper).verify_contracts(&quote, &data).unwrap();

        assert_eq!((data.to.as_str(), data.value.clone(), data.gas_limit.clone()), (deployment.metarouter, BigUint::ZERO, Some("750000".to_string())));
        assert_eq!(
            data.approval.map(|approval| (approval.token, approval.spender, approval.value)),
            Some((usdc.to_checksum(None), metarouter.to_checksum(None), token.value.clone()))
        );
        assert_eq!(commands, vec![CL_SWAP_EXACT_IN_COMMAND, TRANSFER_COMMAND, BALANCE_CHECK_COMMAND, UNWRAP_WETH_COMMAND]);
        assert_eq!(
            SwapExactIn::abi_decode(&inputs[0]).unwrap(),
            SwapExactIn {
                pools: vec![pool],
                tokenIn: usdc,
                amountIn: BalanceSpend::amount(U256::from(100_000_000u64)),
                payerIsUser: true,
                minAmountOut: U256::ZERO,
                recipient: metarouter,
            }
        );
        assert_eq!(
            Transfer::abi_decode(&inputs[1]).unwrap(),
            Transfer {
                token: weth,
                recipient: referral,
                spend: BalanceSpend::share(50)
            }
        );
        assert_eq!(
            BalanceCheck::abi_decode(&inputs[2]).unwrap(),
            BalanceCheck {
                asset: weth,
                owner: metarouter,
                minBalance: min_amount_out
            }
        );
        assert_eq!(
            UnwrapWeth::abi_decode(&inputs[3]).unwrap(),
            UnwrapWeth {
                recipient: token.wallet_address.parse().unwrap(),
                spend: BalanceSpend::all()
            }
        );

        let (cbbtc, usdt): (Address, Address) = (BASE_CBBTC_TOKEN_ID.parse().unwrap(), BASE_USDT_TOKEN_ID.parse().unwrap());
        let disconnected = PoolPath::new(
            PROTOCOL,
            vec![
                TokenPair {
                    token_in: usdc,
                    token_out: cbbtc,
                    fee_tier: FeeTier::Fifty,
                },
                TokenPair {
                    token_in: usdt,
                    token_out: weth,
                    fee_tier: FeeTier::Fifty,
                },
            ],
        );
        let elsewhere = Quote {
            request: QuoteRequest {
                to_asset: BASE_CBBTC_ASSET_ID.clone().into(),
                ..token.clone()
            },
            ..quote
        };
        for refused in [elsewhere, QuoteResult { amount_out, path: disconnected }.into_quote(&token, aero.provider().clone(), false, U256::from(1))] {
            assert_eq!(aero.get_quote_data(&refused, FetchQuoteData::None).await, Err(SwapperError::InvalidRoute));
        }
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        swapper::GemSwapper,
        testkit::{dynode_provider, mock_quote},
    };
    use alloy_sol_types::SolValue;
    use gem_evm::{
        jsonrpc::{BlockParameter, TransactionObject},
        uniswap::contracts::aero::{IMetarouter, TRANSFER_COMMAND, Transfer},
    };
    use num_traits::ToPrimitive;
    use primitives::{
        asset_constants::{ARC_EURC_ASSET_ID, BASE_CBBTC_ASSET_ID, BASE_USDC_ASSET_ID, ROBINHOOD_USDG_ASSET_ID},
        swap::EVM_REFERRAL_ADDRESS,
    };

    fn referral_transfer(data: &SwapperQuoteData) -> Option<Transfer> {
        let execute = IMetarouter::executeCall::abi_decode(&decode_hex(&data.data).ok()?).ok()?;
        let referral = eth_address::parse_str(EVM_REFERRAL_ADDRESS).ok()?;
        execute
            .commands
            .iter()
            .zip(&execute.inputs)
            .filter(|(command, _)| **command == TRANSFER_COMMAND)
            .find_map(|(_, input)| Transfer::abi_decode(input).ok().filter(|transfer| transfer.recipient == referral))
    }

    async fn simulate(chain: Chain, wallet: &str, data: &SwapperQuoteData) -> Result<(), SwapperError> {
        let balance = format!("0x{:x}", data.value.to_u128().unwrap());
        let simulation = EthereumRpc::CallWithBalance {
            transaction: TransactionObject {
                value: Some(balance.clone()),
                ..TransactionObject::new_call_with_from(wallet, &data.to, decode_hex(&data.data).unwrap())
            },
            block: BlockParameter::Latest,
            address: wallet.into(),
            balance,
        };
        create_client_with_chain(dynode_provider([chain]), chain)?.request::<String, _>(simulation).await.map_err(SwapperError::compute_quote_error)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_native_swaps_simulate() -> Result<(), SwapperError> {
        let cases = [
            (Chain::Base, BASE_USDC_ASSET_ID.clone(), 10_000_000_000_000_000u128),
            (Chain::Robinhood, ROBINHOOD_USDG_ASSET_ID.clone(), 10_000_000_000_000_000u128),
            (Chain::Arc, ARC_EURC_ASSET_ID.clone(), 10_000_000_000_000_000_000u128),
        ];
        for (chain, to_asset, value) in cases {
            let provider = Aero::new(dynode_provider([chain]));
            let request = QuoteRequest {
                value: BigUint::from(value),
                ..mock_quote(AssetId::from_chain(chain).into(), to_asset.into())
            };
            let quote = provider.get_quote(&request).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            (&provider as &dyn Swapper).verify_contracts(&quote, &data)?;
            println!("{chain} native -> {} through {:?}, fee {:?}", quote.to_value, quote.data.routes, referral_transfer(&data));

            assert_eq!((data.to.as_str(), data.approval.clone(), data.gas_limit.clone()), (get_aero_deployment_by_chain(&chain).unwrap().metarouter, None, None));
            assert!(referral_transfer(&data).is_some());
            simulate(chain, &request.wallet_address, &data).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_base_cbbtc_to_usdc_quote() -> Result<(), SwapperError> {
        let swapper = GemSwapper::new(dynode_provider([Chain::Base]));
        let request = QuoteRequest {
            value: BigUint::from(10_000_000u64),
            ..mock_quote(BASE_CBBTC_ASSET_ID.clone().into(), BASE_USDC_ASSET_ID.clone().into())
        };
        for provider in [SwapperProvider::UniswapV3, SwapperProvider::Aerodrome] {
            match swapper.get_quote_by_provider(&provider, &request).await {
                Ok(quote) => println!("0.1 cbBTC -> {} USDC through {provider:?}", quote.to_value),
                Err(error) => println!("0.1 cbBTC -> {provider:?} {error:?}"),
            }
        }
        let quote = swapper.get_quote_by_provider(&SwapperProvider::Aero, &request).await?;
        let data = swapper.get_quote_data(&quote, FetchQuoteData::None).await?;
        println!("0.1 cbBTC -> {} USDC through Aero {:?}", quote.to_value, quote.data.routes);

        let metarouter = get_aero_deployment_by_chain(&Chain::Base).unwrap().metarouter;
        assert_eq!(data.approval.map(|approval| approval.spender.to_lowercase()), Some(metarouter.to_lowercase()));
        assert_eq!(data.gas_limit, Some((SWAP_GAS_LIMIT + POOL_GAS_LIMIT * quote.data.routes.len() as u64).to_string()));
        Ok(())
    }
}
