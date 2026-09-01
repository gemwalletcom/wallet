use std::{collections::HashSet, fmt::Debug, sync::Arc, time::Duration};

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use primitives::{AssetId, Chain, ChainType, MINUTE, swap::ApprovalData};

use num_bigint::BigInt;

use super::{
    DUST_THRESHOLD_MULTIPLIER, QUOTE_INTERVAL, QUOTE_MINIMUM, QUOTE_QUANTITY, THORChainNetwork,
    asset::{THORChainAsset, value_to},
    chain::ChainName,
    model::{AsgardVault, InboundAddress, InboundAddressesExt, RouteData},
    quote_data_mapper, quote_mapper, swap_mapper,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperQuoteData, approval::check_approval_erc20, cross_chain::VaultAddresses, fees::default_referral_fees, route_cache::Cache, thorchain::client::ThorChainSwapClient,
};

const INBOUND_ADDRESS_CACHE_TTL: Duration = MINUTE.saturating_mul(5);

#[derive(Debug)]
pub struct ThorChain<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    network: THORChainNetwork,
    client: ThorChainSwapClient<C>,
    inbound_address_cache: Cache<(), Vec<InboundAddress>>,
}

impl<C> ThorChain<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_client(swap_client: ThorChainSwapClient<C>, rpc_provider: Arc<dyn RpcProvider>, network: THORChainNetwork) -> Self {
        Self {
            provider: ProviderType::new(network.provider()),
            rpc_provider,
            network,
            client: swap_client,
            inbound_address_cache: Cache::new(INBOUND_ADDRESS_CACHE_TTL),
        }
    }

    async fn get_inbound_addresses(&self) -> Result<Vec<InboundAddress>, SwapperError> {
        if let Some(addresses) = self.inbound_address_cache.get(&()) {
            return Ok(addresses);
        }
        let addresses = self.client.get_inbound_addresses().await?;
        self.inbound_address_cache.put((), addresses.clone());
        Ok(addresses)
    }

    fn map_quote_error(&self, error: SwapperError, decimals: i32) -> SwapperError {
        match error {
            SwapperError::InputAmountError { min_amount: Some(min) } => SwapperError::InputAmountError {
                min_amount: Some(value_to(&min, decimals).to_string()),
            },
            other => other,
        }
    }
}

impl ThorChain<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Thorchain).expect("Failed to get Thorchain endpoint");
        Self::with_endpoint(endpoint, rpc_provider, THORChainNetwork::Thorchain)
    }

    pub fn new_mayachain(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Mayachain).expect("Failed to get Mayachain endpoint");
        Self::with_endpoint(endpoint, rpc_provider, THORChainNetwork::Mayachain)
    }

    fn with_endpoint(endpoint: String, rpc_provider: Arc<dyn RpcProvider>, network: THORChainNetwork) -> Self {
        let swap_client = ThorChainSwapClient::new(RpcClient::new(endpoint, rpc_provider.clone()), network);
        Self::with_client(swap_client, rpc_provider, network)
    }
}

#[async_trait]
impl<C> Swapper for ThorChain<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        ChainName::supported(self.network)
            .iter()
            .map(|name| SwapperChainAsset::Assets(name.chain(), name.token_assets().into_iter().map(|asset| asset.id).collect()))
            .collect()
    }

    fn amount_mode(&self, request: &QuoteRequest) -> SwapAmountMode {
        match request.from_asset.chain().chain_type() {
            ChainType::Ethereum => SwapAmountMode::Fixed,
            _ => SwapAmountMode::Flexible,
        }
    }

    async fn preload_routes(&self, _from_asset: &AssetId, _to_asset: &AssetId) {
        _ = self.get_inbound_addresses().await;
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        let vaults = self.client.get_asgard_vaults().await?;
        let asgard_addresses: HashSet<String> = AsgardVault::all_addresses(self.network, &vaults).into_iter().collect();
        let router_addresses: HashSet<String> = self.network.router_addresses().iter().map(|address| address.to_string()).collect();

        let deposit: Vec<String> = asgard_addresses.union(&router_addresses).cloned().collect();
        let send: Vec<String> = asgard_addresses.into_iter().collect();

        Ok(VaultAddresses { deposit, send })
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_asset = THORChainAsset::from_asset_id(self.network, &request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(self.network, &request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?;

        let value = super::asset::value_from(&request.value.to_string(), from_asset.decimals as i32);
        let inbound_addresses = self.get_inbound_addresses().await?;
        let from_inbound_address = inbound_addresses.inbound_address_for_asset(self.network, &from_asset)?;
        let to_inbound_address = inbound_addresses.inbound_address_for_asset(self.network, &to_asset)?;

        if [from_inbound_address, to_inbound_address].into_iter().flatten().any(|address| !address.is_swap_available()) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        if let Some(address) = from_inbound_address {
            let min_value = min_value(&address.dust_threshold);
            if min_value > value {
                return Err(SwapperError::InputAmountError {
                    min_amount: Some(value_to(&min_value.to_string(), from_asset.decimals as i32).to_string()),
                });
            }
        }

        let fee = default_referral_fees().thorchain;
        let quote = self
            .client
            .get_quote(
                from_asset.clone(),
                to_asset.clone(),
                value.to_string(),
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps.into(),
            )
            .await
            .map_err(|e| self.map_quote_error(e, from_asset.decimals as i32))?;

        if quote.recommended_min_amount_in > value {
            return Err(SwapperError::InputAmountError {
                min_amount: Some(value_to(&quote.recommended_min_amount_in.to_string(), from_asset.decimals as i32).to_string()),
            });
        }

        let to_value = super::asset::value_to(&quote.expected_amount_out, to_asset.decimals as i32);
        let inbound_address = RouteData::get_inbound_address(self.network, &from_asset, quote.inbound_address.clone())?;
        let route_data = RouteData {
            router_address: quote.router.clone(),
            inbound_address,
        };

        let quote = Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: to_value.magnitude().clone(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(quote_mapper::map_eta_in_seconds(request.to_asset.chain(), quote.total_swap_seconds)),
        };

        Ok(quote)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let fee = default_referral_fees().thorchain;
        let from_asset = THORChainAsset::from_asset_id(self.network, &quote.request.from_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(self.network, &quote.request.to_asset.id).ok_or(SwapperError::NotSupportedAsset)?;
        let memo_asset_name = if to_asset.is_token() {
            to_asset.quote_asset_name()
        } else {
            to_asset.chain.short_name().to_string()
        };

        let memo = to_asset.swap_memo(
            &memo_asset_name,
            quote.request.destination_address.clone(),
            QUOTE_MINIMUM,
            QUOTE_INTERVAL,
            QUOTE_QUANTITY,
            fee.address,
            fee.bps,
        );

        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: RouteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let value = quote.from_value.clone();

        let approval: Option<ApprovalData> = {
            if from_asset.use_evm_router() {
                let router_address = route_data.router_address.clone().ok_or(SwapperError::InvalidRoute)?;
                let from_amount: U256 = value.to_string().parse().map_err(SwapperError::from)?;
                let token_id = from_asset.token_id.clone().ok_or(SwapperError::NotSupportedAsset)?;
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    token_id,
                    router_address,
                    from_amount,
                    self.rpc_provider.clone(),
                    &from_asset.chain.chain(),
                )
                .await?
                .approval_data()
            } else {
                None
            }
        };

        let data = quote_data_mapper::map_quote_data(&from_asset, &route_data, quote.request.from_asset.asset_id().token_id, value, memo, approval);

        Ok(data)
    }

    async fn get_swap_result(&self, _chain: Chain, hash: &str) -> Result<SwapResult, SwapperError> {
        let hash = hash.strip_prefix("0x").unwrap_or(hash).to_uppercase();
        let response = self.client.get_transaction_status(&hash).await?;
        Ok(swap_mapper::map_swap_result(&response, self.network))
    }
}

fn min_value(dust_threshold: &BigInt) -> BigInt {
    dust_threshold * DUST_THRESHOLD_MULTIPLIER
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::{Options, SwapperQuoteAsset, alien::mock::ProviderMock, testkit::mock_bitcoin_max_quote, testkit::mock_quote};
    use gem_client::testkit::MockClient;
    use primitives::asset_constants::{ARBITRUM_USDC_ASSET_ID, THORCHAIN_TCY_ASSET_ID};

    fn thorchain() -> ThorChain<MockClient> {
        ThorChain::with_client(
            ThorChainSwapClient::new(MockClient::new(), THORChainNetwork::Thorchain),
            Arc::new(ProviderMock::new(String::new())),
            THORChainNetwork::Thorchain,
        )
    }

    #[test]
    fn test_map_quote_error() {
        let thorchain = thorchain();
        let cases = [(18, "6614750000000000"), (8, "661475"), (6, "6614")];

        for (decimals, expected) in cases {
            let error = SwapperError::InputAmountError {
                min_amount: Some("661475".to_string()),
            };
            assert_eq!(
                thorchain.map_quote_error(error, decimals),
                SwapperError::InputAmountError {
                    min_amount: Some(expected.to_string())
                }
            );
        }

        assert_eq!(
            thorchain.map_quote_error(SwapperError::InputAmountError { min_amount: None }, 18),
            SwapperError::InputAmountError { min_amount: None }
        );
        assert_eq!(thorchain.map_quote_error(SwapperError::NotSupportedAsset, 18), SwapperError::NotSupportedAsset);
    }

    #[test]
    fn test_min_value() {
        assert_eq!(min_value(&BigInt::from(10000)), BigInt::from(20000));
        assert_eq!(min_value(&BigInt::from(0)), BigInt::from(0));
        assert_eq!(min_value(&BigInt::from(50000)), BigInt::from(100000));
    }

    #[test]
    fn test_supported_assets_contains_zcash() {
        let provider = Arc::new(ProviderMock::new(String::new()));
        let swapper = ThorChain::new(provider);

        let supported = swapper.supported_assets();
        let has_zcash = supported.iter().any(|asset| match asset {
            SwapperChainAsset::Assets(chain, _) => *chain == Chain::Zcash,
            SwapperChainAsset::All(chain) => *chain == Chain::Zcash,
        });

        assert!(has_zcash);
    }

    #[test]
    fn test_mayachain_supported_assets() {
        let provider = Arc::new(ProviderMock::new(String::new()));
        let thorchain = ThorChain::new(provider.clone());
        let mayachain = ThorChain::new_mayachain(provider);

        assert!(!thorchain.supported_assets().iter().any(|asset| asset.get_chain() == Chain::Arbitrum));
        assert!(mayachain.supported_assets().iter().any(|asset| asset.get_chain() == Chain::Arbitrum));
        assert!(
            mayachain
                .supported_assets()
                .iter()
                .any(|asset| { matches!(asset, SwapperChainAsset::Assets(chain, assets) if *chain == Chain::Arbitrum && assets.contains(&ARBITRUM_USDC_ASSET_ID)) })
        );
        assert!(mayachain.supported_assets().iter().any(|asset| asset.get_chain() == Chain::Cardano));
        assert!(
            mayachain
                .supported_assets()
                .iter()
                .any(|asset| { matches!(asset, SwapperChainAsset::Assets(chain, assets) if *chain == Chain::Thorchain && !assets.contains(&THORCHAIN_TCY_ASSET_ID)) })
        );
    }
    #[test]
    fn test_amount_mode_fixed_for_evm_flexible_for_deposit_chains() {
        let provider = ThorChain::with_endpoint("http://localhost".into(), Arc::new(ProviderMock::new(String::new())), THORChainNetwork::Thorchain);

        let bitcoin_request = mock_bitcoin_max_quote(SwapperQuoteAsset::from(Chain::Solana.as_asset_id()));
        assert_eq!(provider.amount_mode(&bitcoin_request), SwapAmountMode::Flexible);

        let mut ethereum_request = mock_bitcoin_max_quote(SwapperQuoteAsset::from(Chain::Solana.as_asset_id()));
        ethereum_request.from_asset = SwapperQuoteAsset::from(Chain::Ethereum.as_asset_id());
        assert_eq!(provider.amount_mode(&ethereum_request), SwapAmountMode::Fixed);
    }

    #[tokio::test]
    async fn test_get_quote_rejects_halted_source_and_destination_chains() {
        let inbound_address_calls = Arc::new(AtomicUsize::new(0));
        let calls = inbound_address_calls.clone();
        let client = MockClient::new().with_get(move |path| {
            calls.fetch_add(1, Ordering::Relaxed);
            assert_eq!(path, "/thorchain/inbound_addresses");
            Ok(include_str!("testdata/inbound_addresses_bsc_halted.json").as_bytes().to_vec())
        });
        let swapper = ThorChain::with_client(
            ThorChainSwapClient::new(client, THORChainNetwork::Thorchain),
            Arc::new(ProviderMock::new(String::new())),
            THORChainNetwork::Thorchain,
        );
        let bsc = SwapperQuoteAsset::from(Chain::SmartChain.as_asset_id());
        let bitcoin = SwapperQuoteAsset::from(Chain::Bitcoin.as_asset_id());
        swapper.preload_routes(&bsc.asset_id(), &bitcoin.asset_id()).await;

        let source_error = swapper.get_quote(&mock_quote(bsc.clone(), bitcoin.clone())).await.unwrap_err();
        let destination_error = swapper.get_quote(&mock_quote(bitcoin, bsc)).await.unwrap_err();

        assert_eq!(source_error, SwapperError::NoQuoteAvailable);
        assert_eq!(destination_error, SwapperError::NoQuoteAvailable);
        assert_eq!(inbound_address_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_get_quote_data_uses_quote_from_value() {
        let provider = Arc::new(ProviderMock::new(String::new()));
        let swapper = ThorChain::new(provider);
        let route_data = RouteData {
            router_address: None,
            inbound_address: "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML".to_string(),
        };
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(Chain::Zcash.as_asset_id()),
            to_asset: SwapperQuoteAsset::from(Chain::Bitcoin.as_asset_id()),
            wallet_address: "t1sender".to_string(),
            destination_address: "bc1qdestination".to_string(),
            value: BigUint::from(11000000u64),
            options: Options::default(),
        };
        let quote = Quote {
            from_value: BigUint::from(10000000u64),
            min_from_value: None,
            to_value: BigUint::from(1u64),
            data: ProviderData {
                provider: swapper.provider().clone(),
                routes: vec![Route {
                    input: Chain::Zcash.as_asset_id(),
                    output: Chain::Bitcoin.as_asset_id(),
                    route_data: serde_json::to_string(&route_data).unwrap(),
                }],
                slippage_bps: 50,
            },
            request,
            eta_in_seconds: None,
        };

        let data = swapper.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(data.to, "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML");
        assert_eq!(data.value, BigUint::from(10000000u64));
        assert_eq!(data.memo, Some("=:b:bc1qdestination:0/1/0:g1:50".to_string()));
    }

    #[tokio::test]
    async fn test_get_quote_data_uses_mayachain_cardano_memo() {
        let provider = Arc::new(ProviderMock::new(String::new()));
        let swapper = ThorChain::new_mayachain(provider);
        let route_data = RouteData {
            router_address: None,
            inbound_address: "addr1v9mr3ts7yr83jphtmfc3256x0ncsj7tayvschr200jt94fg4a96ur".to_string(),
        };
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(Chain::Cardano.as_asset_id()),
            to_asset: SwapperQuoteAsset::from(Chain::Bitcoin.as_asset_id()),
            wallet_address: "addr1qsender".to_string(),
            destination_address: "bc1qdestination".to_string(),
            value: BigUint::from(2000000u64),
            options: Options::default(),
        };
        let quote = Quote {
            from_value: BigUint::from(2000000u64),
            min_from_value: None,
            to_value: BigUint::from(1u64),
            data: ProviderData {
                provider: swapper.provider().clone(),
                routes: vec![Route {
                    input: Chain::Cardano.as_asset_id(),
                    output: Chain::Bitcoin.as_asset_id(),
                    route_data: serde_json::to_string(&route_data).unwrap(),
                }],
                slippage_bps: 50,
            },
            request,
            eta_in_seconds: None,
        };

        let data = swapper.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(data.to, "addr1v9mr3ts7yr83jphtmfc3256x0ncsj7tayvschr200jt94fg4a96ur");
        assert_eq!(data.value, BigUint::from(2000000u64));
        assert_eq!(data.memo, Some("=:b:bc1qdestination:0/1/0:g1:50".to_string()));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperProvider, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, testkit::mock_quote};
    use num_bigint::BigUint;
    use primitives::swap::SwapStatus;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_thorchain_quote_trx_to_bnb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Tron.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::SmartChain.as_asset_id());
        let request = mock_quote(from_asset, to_asset);

        let quote = swapper.get_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert!(quote.eta_in_seconds.is_some());
        assert!(!quote.data.routes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_thorchain_quote_rune_to_cosmos() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Thorchain.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::Cosmos.as_asset_id());
        let mut request = mock_quote(from_asset, to_asset);
        request.value = BigUint::from(100000000u64);

        let quote = swapper.get_quote(&request).await?;

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert!(quote.eta_in_seconds.is_some());
        assert!(!quote.data.routes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_thorchain_quote_rejects_below_min_value() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Xrp.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::Thorchain.as_asset_id());
        let mut request = mock_quote(from_asset, to_asset);
        request.value = BigUint::from(1u64);

        let err = swapper.get_quote(&request).await.expect_err("expected error");
        assert!(matches!(err, SwapperError::InputAmountError { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn test_thorchain_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let swapper = ThorChain::new(provider.clone());

        let tx_hash = "324c16cf014cceca1b2e1c078417f736c9833197735b71a4e875bbb3b07b2fe4";
        let result = swapper.get_swap_result(Chain::Doge, tx_hash).await?;

        assert_eq!(result.status, SwapStatus::Completed);

        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, Chain::Doge.as_asset_id());
        assert!(metadata.from_value > BigUint::ZERO);
        assert!(metadata.to_value > BigUint::ZERO);
        assert_eq!(metadata.provider.unwrap(), "thorchain");

        Ok(())
    }

    #[tokio::test]
    async fn test_mayachain_quote_btc_to_eth() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let mayachain = ThorChain::new_mayachain(provider.clone());

        let from_asset = SwapperQuoteAsset::from(Chain::Bitcoin.as_asset_id());
        let to_asset = SwapperQuoteAsset::from(Chain::Ethereum.as_asset_id());
        let mut request = mock_quote(from_asset, to_asset);
        request.value = BigUint::from(5000000u64); // 0.05 BTC (1e8)
        request.destination_address = "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238".to_string();

        let quote = mayachain.get_quote(&request).await?;
        let quote_data = mayachain.get_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote.data.provider.id, SwapperProvider::Mayachain);
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert!(!quote.data.routes.is_empty());
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.memo, Some("=:e:0x1c7d4b196cb0c7b01d743fbc6116a902379c7238:0/1/0:g1:50".to_string()));

        Ok(())
    }
}
