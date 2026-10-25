use crate::ranking::{leaves_no_dust, rank_quotes};
use crate::{
    AssetList, FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, SwapQuoteError, SwapQuotes, SwapResult, SwapResultRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperProviderMode, SwapperQuoteData,
    across, alien::RpcProvider, bridgers, cetus_clmm, chainflip, config::quote_preferences, cross_chain::VaultAddresses, dedust, deepbook, hyperion, hyperliquid, jupiter, mayan, meteora_damm_v2, meteora_dlmm, near_intents, okx, orca,
    panora, provider_group::ProviderGroup, pump_swap, raydium_amm, raydium_clmm, raydium_cpmm, relay, solana_dex, squid, stellar_dex, stonfi, swaps_xyz, thorchain, uniswap, xrp_ledger,
};
use primitives::{AssetId, Chain, EVMChain};
use std::{collections::HashSet, fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct GemSwapper {
    pub(crate) swappers: Vec<Box<dyn Swapper>>,
}

impl GemSwapper {
    fn filter_by_provider_mode(mode: &SwapperProviderMode, from_chain: Chain, to_chain: Chain) -> bool {
        match mode {
            SwapperProviderMode::OnChain => from_chain == to_chain,
            SwapperProviderMode::Bridge | SwapperProviderMode::CrossChain => from_chain != to_chain,
            SwapperProviderMode::OmniChain(chains) => chains.contains(&from_chain) || from_chain != to_chain,
        }
    }

    fn filter_by_supported_chains(supported_chains: Vec<Chain>, from_chain: Chain, to_chain: Chain) -> bool {
        supported_chains.contains(&from_chain) && supported_chains.contains(&to_chain)
    }

    pub fn leaves_no_dust(&self, quote: &Quote) -> bool {
        leaves_no_dust(quote, self.get_swapper_by_provider(&quote.data.provider.id).ok().map(|swapper| swapper.amount_mode(&quote.request)))
    }

    fn get_swapper_by_provider(&self, provider: &SwapperProvider) -> Result<&dyn Swapper, SwapperError> {
        self.swappers.iter().find(|x| x.provider().id == *provider).map(|v| &**v).ok_or(SwapperError::NoAvailableProvider)
    }

    fn gas_limit_with_multiplier(chain: &Chain, gas_limit: String) -> String {
        match (EVMChain::from_chain(*chain), gas_limit.parse::<u64>()) {
            (Some(evm_chain), Ok(value)) if evm_chain.is_zkstack() => value.saturating_mul(2).to_string(),
            _ => gas_limit,
        }
    }
}

impl GemSwapper {
    fn boxed<T: Swapper + 'static>(swapper: T) -> Box<dyn Swapper> {
        Box::new(swapper)
    }

    fn group(id: SwapperProvider, members: Vec<Option<(&'static str, Box<dyn Swapper>)>>) -> Option<Box<dyn Swapper>> {
        let members = members.into_iter().flatten().collect::<Vec<_>>();
        (!members.is_empty()).then(|| Self::boxed(ProviderGroup::new(id, members)))
    }

    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let swappers: Vec<Box<dyn Swapper>> = [
            Self::group(
                SwapperProvider::UniswapV3,
                vec![
                    Some(("uniswap_v3", uniswap::default::boxed_uniswap_v3(rpc_provider.clone()))),
                    Some(("uniswap_v4", uniswap::default::boxed_uniswap_v4(rpc_provider.clone()))),
                ],
            ),
            Some(uniswap::default::boxed_pancakeswap(rpc_provider.clone())),
            thorchain::ThorChain::new(rpc_provider.clone()).map(Self::boxed),
            thorchain::ThorChain::new_mayachain(rpc_provider.clone()).map(Self::boxed),
            jupiter::Jupiter::new(rpc_provider.clone()).map(Self::boxed),
            solana_dex::SolanaDex::new(orca::Orca, rpc_provider.clone()).map(Self::boxed),
            Self::group(
                SwapperProvider::RaydiumClmm,
                vec![
                    solana_dex::SolanaDex::new(raydium_clmm::RaydiumClmm, rpc_provider.clone()).map(|dex| ("raydium_clmm", Self::boxed(dex))),
                    solana_dex::SolanaDex::new(raydium_cpmm::RaydiumCpmm, rpc_provider.clone()).map(|dex| ("raydium_cpmm", Self::boxed(dex))),
                    solana_dex::SolanaDex::new(raydium_amm::RaydiumAmm, rpc_provider.clone()).map(|dex| ("raydium_amm", Self::boxed(dex))),
                ],
            ),
            solana_dex::SolanaDex::new(pump_swap::PumpSwap, rpc_provider.clone()).map(Self::boxed),
            Self::group(
                SwapperProvider::MeteoraDammV2,
                vec![
                    solana_dex::SolanaDex::new(meteora_damm_v2::MeteoraDammV2, rpc_provider.clone()).map(|dex| ("meteora_damm_v2", Self::boxed(dex))),
                    solana_dex::SolanaDex::new(meteora_dlmm::MeteoraDlmm, rpc_provider.clone()).map(|dex| ("meteora_dlmm", Self::boxed(dex))),
                ],
            ),
            Some(Box::new(okx::OkxProvider::new(rpc_provider.clone()))),
            Some(Box::new(across::Across::new(rpc_provider.clone()))),
            Some(Box::new(hyperliquid::Hyperliquid::new(rpc_provider.clone()))),
            Some(uniswap::default::boxed_oku(rpc_provider.clone())),
            Some(uniswap::default::boxed_wagmi(rpc_provider.clone())),
            stonfi::Stonfi::new(rpc_provider.clone()).map(Self::boxed),
            dedust::Dedust::new(rpc_provider.clone()).map(Self::boxed),
            Some(Self::boxed(uniswap::sunswap::SunSwap::new(rpc_provider.clone()))),
            Some(Box::new(mayan::Mayan::new(rpc_provider.clone()))),
            Some(Box::new(panora::Panora::default())),
            hyperion::Hyperion::new(rpc_provider.clone()).map(Self::boxed),
            Some(Self::boxed(near_intents::NearIntents::new(rpc_provider.clone()))),
            Some(Box::new(chainflip::ChainflipProvider::new(rpc_provider.clone()))),
            cetus_clmm::CetusClmm::new(rpc_provider.clone()).map(Self::boxed),
            deepbook::Deepbook::new(rpc_provider.clone()).map(Self::boxed),
            Some(Box::new(relay::Relay::new(rpc_provider.clone()))),
            Some(Box::new(squid::Squid::new(rpc_provider.clone()))),
            swaps_xyz::SwapsXyz::new(rpc_provider.clone()).map(Self::boxed),
            Some(uniswap::default::boxed_aerodrome(rpc_provider.clone())),
            Some(Self::boxed(uniswap::aero::Aero::new(rpc_provider.clone()))),
            Some(Box::new(bridgers::Bridgers::new(rpc_provider.clone()))),
            stellar_dex::StellarDex::new(rpc_provider.clone()).map(Self::boxed),
            xrp_ledger::XrpLedger::new(rpc_provider.clone()).map(Self::boxed),
        ]
        .into_iter()
        .flatten()
        .collect();

        Self { swappers }
    }

    pub fn supported_chains(&self) -> Vec<Chain> {
        self.swappers.iter().flat_map(|x| x.supported_chains()).collect::<HashSet<_>>().into_iter().collect()
    }

    pub fn supported_chains_for_from_asset(&self, asset_id: &AssetId) -> AssetList {
        let asset_ids = self
            .swappers
            .iter()
            .map(|provider| provider.supported_assets())
            .filter(|supported_assets| supported_assets.iter().any(|supported| supported.supports(asset_id)))
            .flatten()
            .flat_map(|supported| match supported {
                SwapperChainAsset::All(_) => vec![],
                SwapperChainAsset::Assets(chain, assets) => [vec![chain.as_asset_id()], assets].concat(),
            })
            .collect();
        AssetList { chains: vec![asset_id.chain], asset_ids }
    }

    fn is_native_mirror_pair(from_asset: &AssetId, to_asset: &AssetId) -> bool {
        from_asset.chain == to_asset.chain && ((from_asset.is_native() && to_asset.is_native_mirror()) || (from_asset.is_native_mirror() && to_asset.is_native()))
    }

    pub fn get_providers_for_request(&self, request: &QuoteRequest) -> Result<Vec<ProviderType>, SwapperError> {
        if request.from_asset.id == request.to_asset.id || Self::is_native_mirror_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id()) {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let providers: Vec<ProviderType> = self
            .swappers
            .iter()
            .filter(|x| Self::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
            .filter(|x| Self::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
            .map(|x| x.provider().clone())
            .collect();
        if providers.is_empty() {
            return Err(SwapperError::NoAvailableProvider);
        }
        Ok(providers)
    }

    pub async fn get_quotes(&self, request: &QuoteRequest) -> Result<SwapQuotes, SwapperError> {
        let providers = self.get_providers_for_request(request)?;
        let quote_results = futures::future::join_all(providers.iter().map(|provider| async move { (provider.id, self.get_provider_quote(&provider.id, request).await) })).await;

        let mut quotes = Vec::new();
        let mut errors = Vec::new();
        for (provider, result) in quote_results {
            match result {
                Ok(quote) => quotes.push(quote),
                Err(error) => errors.push(SwapQuoteError::new(Some(provider.id().to_string()), error)),
            }
        }
        Ok(SwapQuotes {
            quotes: rank_quotes(request, quotes, &quote_preferences(), |quote| self.leaves_no_dust(quote)),
            errors,
        })
    }

    pub async fn get_provider_quote(&self, provider: &SwapperProvider, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let swapper = self.get_swapper_by_provider(provider)?;
        let request = Self::quote_request_for_mode(swapper.amount_mode(request), request)?;
        swapper.preload_routes(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;
        swapper.get_quote(&request).await
    }

    fn quote_request_for_mode(mode: SwapAmountMode, request: &QuoteRequest) -> Result<QuoteRequest, SwapperError> {
        match mode {
            SwapAmountMode::Fixed => Ok(QuoteRequest {
                value: request.value_after_fee_reserve()?,
                ..request.clone()
            }),
            SwapAmountMode::Flexible => Ok(request.clone()),
        }
    }

    pub async fn get_quote_by_provider(&self, provider: &SwapperProvider, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let swapper = self.get_swapper_by_provider(provider)?;
        swapper.preload_routes(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;
        swapper.get_quote(request).await
    }

    pub async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let swapper = self.get_swapper_by_provider(&quote.data.provider.id)?;
        let quote_data = swapper.get_quote_data(quote, data).await?;
        swapper.verify_contracts(quote, &quote_data)?;
        Ok(SwapperQuoteData {
            gas_limit: quote_data.gas_limit.map(|gas_limit| Self::gas_limit_with_multiplier(&quote.request.from_asset.chain(), gas_limit)),
            ..quote_data
        })
    }

    pub async fn get_swap_result(&self, provider: SwapperProvider, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        self.get_swapper_by_provider(&provider)?.get_swap_result(request).await
    }

    pub async fn get_vault_addresses(&self, provider: &SwapperProvider, from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        self.get_swapper_by_provider(provider)?.get_vault_addresses(from_timestamp).await
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {
    use num_bigint::BigUint;

    use std::{collections::BTreeSet, sync::Arc, vec};

    use primitives::{
        AssetId, Chain,
        asset_constants::{ARC_USDC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDC_TOKEN_ID, STABLE_USDT0_ASSET_ID},
        swap::ApprovalData,
    };

    use super::*;
    use crate::{
        Options, SwapperProvider, SwapperQuoteAsset,
        alien::reqwest_provider::NativeProvider,
        testkit::{MockSwapper, mock_quote},
        uniswap::universal_router::{new_pancakeswap, new_uniswap_v3},
    };

    #[test]
    fn test_filter_by_provider_type() {
        let providers = [
            SwapperProvider::UniswapV3,
            SwapperProvider::PancakeswapV3,
            SwapperProvider::Jupiter,
            SwapperProvider::Thorchain,
            SwapperProvider::NearIntents,
            SwapperProvider::Chainflip,
        ];
        let filter = |from_chain, to_chain| {
            providers
                .iter()
                .filter(|x| GemSwapper::filter_by_provider_mode(&ProviderType::new(**x).mode, from_chain, to_chain))
                .cloned()
                .collect::<Vec<_>>()
        };

        assert_eq!(filter(Chain::Ethereum, Chain::Optimism), vec![SwapperProvider::Thorchain, SwapperProvider::NearIntents, SwapperProvider::Chainflip]);

        assert_eq!(
            filter(Chain::Tron, Chain::Tron),
            vec![
                SwapperProvider::UniswapV3,
                SwapperProvider::PancakeswapV3,
                SwapperProvider::Jupiter,
                SwapperProvider::Thorchain,
                SwapperProvider::NearIntents,
                SwapperProvider::Chainflip
            ]
        );

        assert_eq!(filter(Chain::Ethereum, Chain::Ethereum), vec![SwapperProvider::UniswapV3, SwapperProvider::PancakeswapV3, SwapperProvider::Jupiter]);

        assert!(filter(Chain::Near, Chain::Near).contains(&SwapperProvider::NearIntents));
    }

    #[test]
    fn test_filter_by_supported_chains() {
        let provider = Arc::new(NativeProvider::default());
        let swappers: Vec<Box<dyn Swapper>> = vec![
            Box::new(new_uniswap_v3(provider.clone())),
            Box::new(new_pancakeswap(provider.clone())),
            Box::new(thorchain::ThorChain::new(provider.clone()).unwrap()),
            Box::new(thorchain::ThorChain::new_mayachain(provider.clone()).unwrap()),
            Box::new(jupiter::Jupiter::new(provider).unwrap()),
        ];
        let filter = |from_chain, to_chain| {
            swappers
                .iter()
                .filter(|x| GemSwapper::filter_by_provider_mode(&x.provider().mode, from_chain, to_chain))
                .filter(|x| GemSwapper::filter_by_supported_chains(x.supported_chains(), from_chain, to_chain))
                .collect::<Vec<_>>()
        };

        let filtered = filter(Chain::Ethereum, Chain::Optimism);

        assert_eq!(filtered.len(), 0);

        let filtered = filter(Chain::SmartChain, Chain::SmartChain);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered.iter().map(|x| x.provider().id).collect::<BTreeSet<_>>(), BTreeSet::from([SwapperProvider::UniswapV3, SwapperProvider::PancakeswapV3]));

        let filtered = filter(Chain::Solana, Chain::Solana);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider().id, SwapperProvider::Jupiter);

        let filtered = filter(Chain::SmartChain, Chain::Bitcoin);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider().id, SwapperProvider::Thorchain);

        let filtered = filter(Chain::Ethereum, Chain::Mayachain);

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_is_native_mirror_pair() {
        let arc = AssetId::from_chain(Chain::Arc);

        assert!(GemSwapper::is_native_mirror_pair(&arc, &ARC_USDC_ASSET_ID));
        assert!(GemSwapper::is_native_mirror_pair(&ARC_USDC_ASSET_ID, &arc));
        assert!(GemSwapper::is_native_mirror_pair(&AssetId::from_chain(Chain::Stable), &STABLE_USDT0_ASSET_ID));
        assert!(!GemSwapper::is_native_mirror_pair(&ARC_USDC_ASSET_ID, &ETHEREUM_USDC_ASSET_ID));
        assert!(!GemSwapper::is_native_mirror_pair(&ARC_USDC_ASSET_ID, &AssetId::from_chain(Chain::Ethereum)));
    }

    #[tokio::test]
    async fn test_get_quotes_collects_per_provider_errors() {
        let request = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)), SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()));

        let gem_swapper = GemSwapper::mock(vec![
            Box::new(MockSwapper::new(SwapperProvider::UniswapV3, |_| Err(SwapperError::InputAmountError { min_amount: None }))),
            Box::new(MockSwapper::new(SwapperProvider::PancakeswapV3, |_| Err(SwapperError::InputAmountError { min_amount: Some("1264000".into()) }))),
            Box::new(MockSwapper::new(SwapperProvider::Jupiter, |_| Err(SwapperError::NoQuoteAvailable))),
        ]);
        let result = gem_swapper.get_quotes(&request).await.unwrap();
        assert!(result.quotes.is_empty());
        assert_eq!(result.errors.len(), 3);

        let providers: BTreeSet<_> = result.errors.iter().map(|e| e.provider.clone().unwrap()).collect();
        assert_eq!(
            providers,
            BTreeSet::from([SwapperProvider::UniswapV3.id().to_string(), SwapperProvider::PancakeswapV3.id().to_string(), SwapperProvider::Jupiter.id().to_string(),])
        );
        let pancake_error = result.errors.iter().find(|e| e.provider.as_deref() == Some(SwapperProvider::PancakeswapV3.id())).unwrap();
        assert!(pancake_error.error.to_string().contains("1264000"));
    }

    #[tokio::test]
    async fn test_get_quote_data_binds_the_target_and_spender_to_the_provider_contracts() {
        let request = mock_quote(SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()), SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)));
        let router = "0x8feab81d36e7576107d5de0758c1b839be31b4f6";
        let token_approve = "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f";
        let approval = |spender: &str| ApprovalData {
            token: ETHEREUM_USDC_TOKEN_ID.to_string(),
            spender: spender.to_string(),
            value: BigUint::from(1_000_000u64),
            is_unlimited: false,
        };
        let swapper = |to: &str, spender: &str| {
            let data = SwapperQuoteData::new_contract(to.to_string(), BigUint::ZERO, "0x".to_string(), Some(approval(spender)), None);
            GemSwapper::mock(vec![Box::new(
                MockSwapper::new(SwapperProvider::Okx, |request| Ok(Quote::mock_with_request(request)))
                    .with_contracts(vec![router, token_approve])
                    .with_quote_data(data),
            )])
        };
        async fn quote_data(swapper: &GemSwapper, request: &QuoteRequest) -> Result<SwapperQuoteData, SwapperError> {
            let quote = swapper.get_quotes(request).await?.quotes.remove(0);
            swapper.get_quote_data(&quote, FetchQuoteData::None).await
        }

        let bound = swapper(&router.to_uppercase().replace("0X", "0x"), token_approve);
        assert!(quote_data(&bound, &request).await.is_ok(), "a shipped target compares without case");

        let foreign_target = swapper("0x0000000000000000000000000000000000000001", token_approve);
        assert_eq!(quote_data(&foreign_target, &request).await.unwrap_err(), SwapperError::InvalidRoute);

        let foreign_spender = swapper(router, "0x0000000000000000000000000000000000000001");
        assert_eq!(quote_data(&foreign_spender, &request).await.unwrap_err(), SwapperError::InvalidRoute);

        let undeclared = GemSwapper::mock(vec![Box::new(
            MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request))).with_quote_data(SwapperQuoteData::new_contract(
                "0x0000000000000000000000000000000000000001".to_string(),
                BigUint::ZERO,
                "0x".to_string(),
                None,
                None,
            )),
        )]);
        assert!(quote_data(&undeclared, &request).await.is_ok(), "a provider that ships no contracts is not bound");
    }

    #[tokio::test]
    async fn test_get_quote_by_provider_asks_for_the_exact_value_it_is_given() {
        let max = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)), SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()));
        let request = QuoteRequest {
            value: BigUint::from(10u64).pow(18),
            options: Options { use_max_amount: true, ..max.options },
            ..max
        };
        let gem_swapper = GemSwapper::mock(vec![Box::new(MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request))))]);

        let quotes = gem_swapper.get_quotes(&request).await.unwrap().quotes;
        let requoted = gem_swapper.get_quote_by_provider(&SwapperProvider::UniswapV3, &quotes[0].request).await.unwrap();

        assert!(quotes[0].request.value < request.value, "the first quote of a max swap keeps the fee reserve back");
        assert_eq!(requoted.request.value, quotes[0].request.value, "a re-quote of that quote asks for the reserved value as it is, never reserving twice");
    }

    #[tokio::test]
    async fn test_a_max_native_quote_keeps_a_fee_reserve_only_for_a_provider_that_needs_the_exact_amount() {
        let max = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)), SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()));
        let request = QuoteRequest {
            value: BigUint::from(10u64).pow(18),
            options: Options { use_max_amount: true, ..max.options },
            ..max
        };
        let gem_swapper = GemSwapper::mock(vec![
            Box::new(MockSwapper::new(SwapperProvider::UniswapV3, |request| Ok(Quote::mock_with_request(request)))),
            Box::new(MockSwapper::new(SwapperProvider::Jupiter, |request| Ok(Quote::mock_with_request(request))).with_amount_mode(SwapAmountMode::Flexible)),
        ]);

        let quotes = gem_swapper.get_quotes(&request).await.unwrap().quotes;
        let asked = |provider: SwapperProvider| quotes.iter().find(|quote| quote.data.provider.id == provider).unwrap().request.value.clone();
        let reserve = crate::fees::reserved_transaction_fees(Chain::Ethereum).unwrap();

        assert_eq!(asked(SwapperProvider::Jupiter), request.value, "a provider that swaps whatever arrives is asked for everything; the fee comes off at signing");
        assert_eq!(
            asked(SwapperProvider::UniswapV3),
            &request.value - reserve,
            "a contract call spends exactly what it is asked, so the fee reserve comes off first"
        );
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
        time::Instant,
    };

    use futures::{StreamExt, stream};
    use num_bigint::BigUint;
    use primitives::{
        AssetFull, AssetId, Chain, ChainType,
        asset_constants::{ETHEREUM_USDC_ASSET_ID, SOLANA_USDC_ASSET_ID},
        testkit::signer_mock::{TEST_EVM_SENDER, TEST_SOLANA_SENDER, TEST_TON_SENDER, TEST_TRON_SENDER},
    };

    use super::*;
    use crate::{QuoteRequest, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, config::API_BASE_URL, models::Options, testkit::mock_quote};

    const CONTRACT_CHECK_AMOUNT_USD: f64 = 50.0;
    const CONTRACT_CHECK_CONCURRENCY: usize = 4;

    fn test_address(chain: Chain) -> Option<&'static str> {
        match chain.chain_type() {
            ChainType::Ethereum => Some(TEST_EVM_SENDER),
            ChainType::Solana => Some(TEST_SOLANA_SENDER),
            ChainType::Ton => Some(TEST_TON_SENDER),
            ChainType::Tron => Some(TEST_TRON_SENDER),
            _ => None,
        }
    }

    fn bound_routes(swappers: &[Box<dyn Swapper>]) -> Vec<(&dyn Swapper, AssetId, AssetId)> {
        let mut tokens: BTreeMap<Chain, Vec<AssetId>> = BTreeMap::new();
        for (chain, assets) in swappers.iter().flat_map(|swapper| swapper.supported_assets()).filter_map(|supported| match supported {
            SwapperChainAsset::Assets(chain, assets) => Some((chain, assets)),
            SwapperChainAsset::All(_) => None,
        }) {
            let listed = tokens.entry(chain).or_default();
            listed.extend(assets.into_iter().filter(|asset| !listed.contains(asset)).collect::<Vec<_>>());
        }
        let destination = |swapper: &dyn Swapper, from: &AssetId| match swapper.provider().mode {
            SwapperProviderMode::OnChain if from.is_native() => tokens.get(&from.chain)?.first().cloned(),
            SwapperProviderMode::OnChain => Some(from.chain.as_asset_id()),
            _ => swapper.supported_chains().into_iter().find(|chain| *chain != from.chain && test_address(*chain).is_some()).map(|chain| chain.as_asset_id()),
        };
        swappers
            .iter()
            .flat_map(|swapper| swapper.supported_assets().into_iter().map(move |supported| (swapper, supported)))
            .flat_map(|(swapper, supported)| {
                let chain = supported.get_chain();
                let assets = match supported {
                    SwapperChainAsset::All(_) => tokens.get(&chain).cloned().unwrap_or_default(),
                    SwapperChainAsset::Assets(_, assets) => assets,
                };
                std::iter::once(chain.as_asset_id())
                    .chain(assets)
                    .filter(|from| !swapper.contracts(from).targets.is_empty())
                    .filter_map(|from| destination(swapper.as_ref(), &from).map(|to| (swapper.as_ref(), from, to)))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    async fn quote_asset(asset_id: &AssetId) -> Option<(SwapperQuoteAsset, BigUint)> {
        let mut url = reqwest::Url::parse(API_BASE_URL).ok()?;
        url.path_segments_mut().ok()?.extend(["v1", "assets", &asset_id.to_string()]);
        let asset: AssetFull = reqwest::get(url).await.ok()?.json().await.ok()?;
        let value = CONTRACT_CHECK_AMOUNT_USD / asset.price?.price * 10f64.powi(asset.asset.decimals as i32);
        Some((SwapperQuoteAsset::from(&asset.asset), BigUint::from(value as u128)))
    }

    async fn checked_route(swapper: &dyn Swapper, from: &AssetId, to: &AssetId) -> Option<Result<(), String>> {
        let ((from_asset, value), (to_asset, _)) = (quote_asset(from).await?, quote_asset(to).await?);
        let request = QuoteRequest {
            from_asset,
            to_asset,
            wallet_address: test_address(from.chain)?.to_string(),
            destination_address: test_address(to.chain)?.to_string(),
            value,
            options: Options::default(),
        };
        let quote = swapper.get_quote(&request).await.ok()?;
        let data = swapper.get_quote_data(&quote, FetchQuoteData::None).await.ok()?;
        Some(swapper.verify_contracts(&quote, &data).map_err(|_| {
            let spender = data.approval.map(|approval| approval.spender).or(data.permit2.map(|permit2| permit2.spender));
            format!("{} {from} -> {to}: calls {}, spender {spender:?}, data {:.200}", swapper.provider().id.as_ref(), data.to, data.data)
        }))
    }

    #[tokio::test]
    async fn test_every_bound_route_passes_the_contract_check() {
        let swapper = GemSwapper::new(Arc::new(NativeProvider::default().set_debug(false)));
        let routes = bound_routes(&swapper.swappers);
        let results: Vec<_> = stream::iter(&routes).map(|(swapper, from, to)| checked_route(*swapper, from, to)).buffer_unordered(CONTRACT_CHECK_CONCURRENCY).collect().await;
        let refused: Vec<String> = results.iter().flatten().filter_map(|result| result.clone().err()).collect();

        println!("{} of {} bound routes quoted", results.iter().flatten().count(), routes.len());
        assert!(refused.is_empty(), "Confirm refuses these routes, each calling a contract its provider does not ship:\n{}", refused.join("\n"));
    }

    async fn report(swapper: &GemSwapper, request: &QuoteRequest, round: &str) {
        let provider_ids: BTreeSet<_> = swapper.get_providers_for_request(request).unwrap().into_iter().map(|provider| provider.id).collect();
        let timings = swapper.swappers.iter().filter(|swapper| provider_ids.contains(&swapper.provider().id)).map(|provider| async move {
            let started = Instant::now();
            provider.preload_routes(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;
            let outcome = provider.get_quote(request).await;
            (provider.provider().id.id().to_string(), started.elapsed().as_millis(), outcome.is_ok())
        });

        let started = Instant::now();
        let mut timings = futures::future::join_all(timings).await;
        let total = started.elapsed().as_millis();
        timings.sort_by_key(|(_, elapsed, _)| *elapsed);
        for (provider, elapsed, quoted) in &timings {
            println!("{round} quote {provider}: {elapsed}ms {}", if *quoted { "quoted" } else { "no quote" });
        }
        println!("{round} quote round total: {total}ms, slowest decides");
    }

    #[tokio::test]
    async fn test_grouped_providers_build_with_the_member_that_quoted() {
        let swapper = GemSwapper::new(crate::testkit::dynode_provider([Chain::Ethereum, Chain::Solana]));
        let ethereum = QuoteRequest {
            wallet_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            destination_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            value: BigUint::from(100000000000000000u64),
            ..mock_quote(AssetId::from_chain(Chain::Ethereum).into(), ETHEREUM_USDC_ASSET_ID.clone().into())
        };
        let solana = QuoteRequest {
            wallet_address: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".into(),
            destination_address: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".into(),
            value: BigUint::from(1000000000u64),
            ..mock_quote(AssetId::from_chain(Chain::Solana).into(), SOLANA_USDC_ASSET_ID.clone().into())
        };

        for (provider, request, members) in [
            (SwapperProvider::UniswapV3, ethereum, ["uniswap_v3", "uniswap_v4"].as_slice()),
            (SwapperProvider::RaydiumClmm, solana.clone(), ["raydium_clmm", "raydium_cpmm", "raydium_amm"].as_slice()),
            (SwapperProvider::MeteoraDammV2, solana, ["meteora_damm_v2", "meteora_dlmm"].as_slice()),
        ] {
            let quote = swapper.get_quote_by_provider(&provider, &request).await.unwrap();
            println!("{provider:?} quoted by {} for {}", quote.data.provider.protocol_id, quote.to_value);

            assert_eq!(quote.data.provider.id, provider);
            assert!(members.contains(&quote.data.provider.protocol_id.as_str()));
            assert!(!swapper.get_quote_data(&quote, FetchQuoteData::None).await.unwrap().data.is_empty());
        }
    }

    #[tokio::test]
    #[ignore = "timing report without assertions, run manually"]
    async fn test_report_preload_and_quote_durations_per_provider() {
        let swapper = GemSwapper::new(Arc::new(NativeProvider::new().set_debug(false)));

        let mut on_chain = mock_quote(AssetId::from_chain(Chain::Ethereum).into(), ETHEREUM_USDC_ASSET_ID.clone().into());
        on_chain.wallet_address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into();
        on_chain.destination_address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into();
        on_chain.value = BigUint::from(100000000000000000u64);
        on_chain.options.slippage = 100.into();
        report(&swapper, &on_chain, "on-chain cold").await;
        report(&swapper, &on_chain, "on-chain warm").await;

        let cross_chain = QuoteRequest {
            to_asset: AssetId::from_chain(Chain::Solana).into(),
            destination_address: "7v91N7iZ9mNicL8WfG6cgSCKyRXydQjLh6UYBWwm6y1Q".into(),
            ..on_chain.clone()
        };
        report(&swapper, &cross_chain, "cross-chain cold").await;
        report(&swapper, &cross_chain, "cross-chain warm").await;
    }
}
