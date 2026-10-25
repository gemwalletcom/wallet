use super::{
    client,
    commands::{SwapCommands, build_commands},
    constants::{FEE_TIERS, REFERRAL_FEE_BPS, SWAP_ENERGY_LIMIT},
};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::RpcProvider,
    approval::check_permit2_approvals_trc20,
    client_factory::create_tron_client,
    eth_address,
    fees::{ReferralFee, default_referral_fees, subtract_bps},
    uniswap::{
        deadline::get_sig_deadline,
        discovery::{PoolDiscovery, candidate_pairs},
        quote_result::{OutpricedPaths, QuoteResult, best_path_quote},
        routed_asset::{Funding, Protocol, RoutedAsset},
        swap_route::{PoolPath, RouteData, candidate_paths, get_intermediaries},
        v3::path::build_paths_with_routes,
    },
};
use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use futures::future::join_all;
use gem_evm::uniswap::{
    command::{Permit2Permit, encode_commands},
    path::{BasePair, build_pairs},
};
use gem_tron::rpc::client::TronClient;
use num_bigint::BigUint;
use primitives::{
    AssetId, Chain,
    asset_constants::{TRON_USDT_TOKEN_ID, TRON_WTRX_TOKEN_ID},
    contract_constants::{TRON_SUNSWAP_PERMIT2_CONTRACT, TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT},
};
use std::{fmt, str::FromStr, sync::Arc};

type Pair = (Chain, Address);

pub struct SunSwap {
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    pool_discovery: PoolDiscovery,
    outpriced_paths: OutpricedPaths,
}

impl SunSwap {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::SunswapV3),
            rpc_provider,
            pool_discovery: PoolDiscovery::default(),
            outpriced_paths: OutpricedPaths::default(),
        }
    }

    fn referral_fee() -> ReferralFee {
        ReferralFee {
            bps: REFERRAL_FEE_BPS,
            ..default_referral_fees().tron
        }
    }

    fn routed_asset(asset_id: &AssetId) -> Result<RoutedAsset, SwapperError> {
        let (token, funding) = match &asset_id.token_id {
            Some(token_id) => (token_id.as_str(), Funding::Token),
            None => (TRON_WTRX_TOKEN_ID, Funding::Value),
        };
        Ok(RoutedAsset::unscaled(eth_address::parse(Chain::Tron, token)?, funding))
    }

    fn routed_pair(from_asset: &AssetId, to_asset: &AssetId) -> Result<(RoutedAsset, RoutedAsset), SwapperError> {
        if from_asset.chain != Chain::Tron || to_asset.chain != Chain::Tron {
            return Err(SwapperError::NotSupportedChain);
        }
        Ok((Self::routed_asset(from_asset)?, Self::routed_asset(to_asset)?))
    }

    fn routed_request(request: &QuoteRequest) -> Result<(RoutedAsset, RoutedAsset, U256), SwapperError> {
        let (input, output) = Self::routed_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id())?;
        let amount_in = U256::from_str(&request.value.to_string()).map_err(SwapperError::from)?;
        Ok((input, output, amount_in))
    }

    fn base_pair() -> Result<BasePair, SwapperError> {
        Ok(BasePair {
            primary: eth_address::parse(Chain::Tron, TRON_WTRX_TOKEN_ID)?,
            stables: vec![eth_address::parse(Chain::Tron, TRON_USDT_TOKEN_ID)?],
            alternatives: vec![],
        })
    }

    async fn preload_pools(&self, client: &TronClient<RpcClient>, token_in: Address, token_out: Address) -> Result<(), SwapperError> {
        let pairs = candidate_pairs(token_in, token_out, get_intermediaries(&token_in, &token_out, &Self::base_pair()?));
        let pools = self.pool_discovery.missing_pools(Chain::Tron, &pairs, &FEE_TIERS);
        let discovered = join_all(pools.iter().map(|pool| async move { client::pool_exists(client, pool).await.map(|exists| (pool.clone(), exists)) })).await;
        self.pool_discovery.record_pools(Chain::Tron, &discovered.into_iter().filter_map(Result::ok).collect::<Vec<_>>());
        Ok(())
    }

    async fn quote_paths(&self, client: &TronClient<RpcClient>, pair: (&Pair, &Pair), amount_in: U256, paths: Vec<PoolPath>) -> Result<QuoteResult, SwapperError> {
        let quotes = join_all(paths.iter().map(|path| client::quote_exact_input(client, build_pairs(&path.pairs), amount_in))).await;
        let outcomes = paths.into_iter().zip(quotes).filter_map(|(path, quote)| Some((path, quote.ok()?))).collect();
        best_path_quote(&self.outpriced_paths, pair, amount_in.saturating_to(), outcomes)
    }
}

impl fmt::Debug for SunSwap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SunSwap").finish()
    }
}

#[async_trait]
impl Swapper for SunSwap {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Tron)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        let (Ok((input, output)), Ok(client)) = (Self::routed_pair(from_asset, to_asset), create_tron_client(self.rpc_provider.clone())) else {
            return;
        };
        _ = self.preload_pools(&client, input.address, output.address).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let (input, output, amount_in) = Self::routed_request(request)?;
        let client = create_tron_client(self.rpc_provider.clone())?;
        let swap_amount = subtract_bps(&amount_in, Self::referral_fee().bps);

        _ = self.preload_pools(&client, input.address, output.address).await;
        let paths = candidate_paths(&input.address, &output.address, &FEE_TIERS, &Self::base_pair()?)
            .into_iter()
            .filter(|pairs| self.pool_discovery.path_may_exist(Chain::Tron, pairs))
            .map(|pairs| PoolPath::new(Protocol::V3, pairs))
            .collect::<Vec<_>>();
        let (from, to) = ((Chain::Tron, input.address), (Chain::Tron, output.address));
        let quote = self
            .outpriced_paths
            .quote_live_first((&from, &to), swap_amount.saturating_to(), paths, PoolPath::clone, |paths| self.quote_paths(&client, (&from, &to), swap_amount, paths))
            .await?;

        Ok(quote.into_quote(request, self.provider.clone(), true, U256::from(1)))
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let (input, output, amount_in) = Self::routed_request(request)?;
        let route_data: RouteData = quote.route_data()?;
        let min_amount_out = U256::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;
        let permit = data.permit2_data().map(Permit2Permit::try_from).transpose()?;

        let (approval, permit2) = match input.funding {
            Funding::Token => {
                let token = eth_address::token_id(Chain::Tron, &input.address);
                check_permit2_approvals_trc20(
                    &request.wallet_address,
                    &token,
                    amount_in,
                    (TRON_SUNSWAP_PERMIT2_CONTRACT, TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT),
                    permit.is_some(),
                    self.rpc_provider.clone(),
                )
                .await?
            }
            Funding::Value | Funding::RouterBalance => (None, None),
        };
        let commands = build_commands(
            &input,
            &output,
            SwapCommands {
                recipient: eth_address::parse(Chain::Tron, &request.wallet_address)?,
                amount_in,
                min_amount_out,
                path: build_paths_with_routes(&quote.data.routes)?,
                permit,
                referral: Self::referral_fee(),
            },
        )?;
        let calldata = encode_commands(&commands, U256::from(get_sig_deadline()));
        let value = match input.funding {
            Funding::Value => request.value.clone(),
            Funding::Token | Funding::RouterBalance => BigUint::ZERO,
        };
        let gas_limit = (approval.is_some() || permit2.is_some()).then(|| SWAP_ENERGY_LIMIT.to_string());

        Ok(SwapperQuoteData {
            permit2,
            ..SwapperQuoteData::new_contract(TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT.to_string(), value, hex::encode(calldata), approval, gas_limit)
        })
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset, testkit::dynode_provider};
    use gem_tron::models::TriggerSmartContractData;
    use num_traits::ToPrimitive;
    use primitives::asset_constants::TRON_USDT_ASSET_ID;

    const TRX_HOLDER: &str = "TV6MuMXfmLbBqPZvBHdwFsDnQeVfnmiuSi";
    const PERMIT2_USDT_HOLDERS: [&str; 4] = [
        "TK2nJVdoyfBL2Htee1NLsiAU2QnuZzCBAQ",
        "TD59KVhr1JoYGrSh9Qos6EzBoSgbDk3BBG",
        "TVbpqADbMuXzhDFhWuFyMYhhqwwvSs8SyN",
        "TSAHsqJtAG7JjNeM12kWLbSpsGRH98TqKj",
    ];
    const SUN_TOKEN_ID: &str = "TSSMHYeV2uE9qYH95DqyoCuNCzEL1NvU3S";

    fn request(from_asset: AssetId, to_asset: AssetId, value: u64) -> QuoteRequest {
        request_from(TRX_HOLDER, from_asset, to_asset, value)
    }

    fn request_from(wallet: &str, from_asset: AssetId, to_asset: AssetId, value: u64) -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(from_asset),
            to_asset: SwapperQuoteAsset::from(to_asset),
            wallet_address: wallet.to_string(),
            destination_address: wallet.to_string(),
            value: BigUint::from(value),
            options: Options::default(),
        }
    }

    async fn simulate(rpc_provider: Arc<dyn RpcProvider>, owner: &str, data: &SwapperQuoteData) -> Result<u64, SwapperError> {
        let contract = TriggerSmartContractData {
            contract_address: data.to.clone(),
            data: data.data.clone(),
            owner_address: owner.to_string(),
            fee_limit: None,
            call_value: data.value.to_u64().filter(|value| *value > 0),
        };
        create_tron_client(rpc_provider)?.estimate_energy_with_data(&contract).await.map_err(SwapperError::transaction_error)
    }

    async fn funded_permit2_holder(rpc_provider: Arc<dyn RpcProvider>, value: u64) -> Result<Option<&'static str>, SwapperError> {
        let client = create_tron_client(rpc_provider)?;
        for holder in PERMIT2_USDT_HOLDERS {
            let parameter = hex::encode(eth_address::parse(Chain::Tron, holder)?.into_word());
            let balance = client.trigger_constant_contract(TRON_USDT_TOKEN_ID, "balanceOf(address)", &parameter).await.map_err(SwapperError::transaction_error)?;
            if U256::from_str_radix(&balance, 16).map_err(SwapperError::transaction_error)? >= U256::from(value) {
                return Ok(Some(holder));
            }
        }
        Ok(None)
    }

    #[tokio::test]
    async fn test_sunswap_trx_to_usdt_simulates() -> Result<(), SwapperError> {
        let rpc_provider = dynode_provider([Chain::Tron]);
        let provider = SunSwap::new(rpc_provider.clone());
        let quote = provider.get_quote(&request(AssetId::from_chain(Chain::Tron), TRON_USDT_ASSET_ID.clone(), 100_000_000)).await?;
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        (&provider as &dyn Swapper).verify_contracts(&quote, &data)?;
        let energy = simulate(rpc_provider, TRX_HOLDER, &data).await?;
        println!("100 TRX -> {} USDT, routes {:?}, energy {energy}", quote.to_value, quote.data.routes);

        assert!(quote.to_value > BigUint::from(10_000_000u64));
        assert_eq!(data.value, BigUint::from(100_000_000u64));
        assert!(data.approval.is_none() && data.permit2.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_sunswap_usdt_to_trx_asks_for_permit2() -> Result<(), SwapperError> {
        let provider = SunSwap::new(dynode_provider([Chain::Tron]));
        let quote = provider.get_quote(&request(TRON_USDT_ASSET_ID.clone(), AssetId::from_chain(Chain::Tron), 10_000_000)).await?;
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        (&provider as &dyn Swapper).verify_contracts(&quote, &data)?;
        println!("10 USDT -> {} sun, approval {:?}, permit2 {:?}", quote.to_value, data.approval, data.permit2);

        assert!(quote.to_value > BigUint::from(10_000_000u64));
        assert_eq!(data.value, BigUint::ZERO);
        assert!(data.approval.is_some() || data.permit2.is_some());
        assert_eq!(data.gas_limit, Some(SWAP_ENERGY_LIMIT.to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_sunswap_usdt_to_trx_simulates_with_a_permit2_allowance() -> Result<(), SwapperError> {
        let rpc_provider = dynode_provider([Chain::Tron]);
        let provider = SunSwap::new(rpc_provider.clone());
        let Some(holder) = funded_permit2_holder(rpc_provider.clone(), 1_000_000).await? else {
            panic!("every PERMIT2_USDT_HOLDERS wallet holds less than 1 USDT; replace them with wallets that keep USDT and a never-expiring SunSwap Permit2 allowance");
        };
        let quote = provider.get_quote(&request_from(holder, TRON_USDT_ASSET_ID.clone(), AssetId::from_chain(Chain::Tron), 1_000_000)).await?;
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        (&provider as &dyn Swapper).verify_contracts(&quote, &data)?;
        let energy = simulate(rpc_provider, holder, &data).await?;
        println!("1 USDT -> {} sun from {holder} with an existing Permit2 allowance, energy {energy}", quote.to_value);

        assert!(
            data.approval.is_none() && data.permit2.is_none(),
            "{holder} no longer has an unlimited, never-expiring USDT Permit2 allowance for the SunSwap router"
        );
        assert_eq!(data.gas_limit, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_sunswap_usdt_to_sun_routes_through_a_base_token() -> Result<(), SwapperError> {
        let provider = SunSwap::new(dynode_provider([Chain::Tron]));
        let quote = provider.get_quote(&request(TRON_USDT_ASSET_ID.clone(), AssetId::from_token(Chain::Tron, SUN_TOKEN_ID), 10_000_000)).await?;
        println!("10 USDT -> {} SUN, routes {:?}", quote.to_value, quote.data.routes);

        assert!(quote.to_value > BigUint::ZERO);
        Ok(())
    }
}
